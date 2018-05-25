use alloc::heap::{AllocErr, Layout};
use std;
use std::mem;
use allocator::util::*;
use allocator::linked_list::LinkedList;
use allocator::page::{
    PGSIZE, Page, PPN, KERNEL_PAGES, NPAGE, 
    MAXPA, pa2page, ATTRIB_AP_RW_ALL, 
    PTE_ADDR, PTE_V, PADDR};
use allocator::alloc_pages;
use allocator;
use mm::pmm::page_insert;
use mm::pmm::{user_pgdir_alloc_page,pgdir_alloc_page};
use mm::vm::{get_pte};
use process::process::utils::memcpy;
use ALLOCATOR;
use alloc::allocator::Alloc;

use console::kprintln;

pub static mut BACKUP_ALLOCATOR :Allocator = Allocator {
    base_paddr: 0,
    base_page: 0,
    free_list: LinkedList::new(),
    n_free: 0,
};

pub static mut USER_ALLOCATOR :Allocator = Allocator {
    base_paddr: 0,
    base_page: 0,
    free_list: LinkedList::new(),
    n_free: 0,
};

/// A "bump" allocator: allocates memory by bumping a pointer; never frees.
#[derive(Copy, Clone, Debug)]
pub struct Allocator {
    free_list: LinkedList,
    n_free: u32,
    base_page: usize,
    pub base_paddr: usize,
}

impl Allocator {
    pub fn new() -> Allocator {
        Self {
            base_paddr: 0,
            base_page: 0,
            free_list: LinkedList::new(),
            n_free: 0,
        }
    }

    pub fn init_user(&mut self, pgdir: *const usize) {
        self.base_page = unsafe{ (MAXPA as *mut Page).sub(MAXPA / PGSIZE) as *mut usize as usize };
        self.base_page = align_down(self.base_page, PGSIZE);
        kprintln!("base_page: {:x}", self.base_page);

        let npage = self.base_page / PGSIZE;
        let n_phy_page = (MAXPA - self.base_page) / PGSIZE;
        kprintln!("unnessasery page: {}", n_phy_page);

        let page_pa = match alloc_pages(n_phy_page) {
            Ok(paddr) => { paddr as *const usize},
            Err(_) => { 
                panic!("Exausted!");
                return; 
            }
        };
        
        kprintln!("physical page addr: {:x}", page_pa as usize);        

        let mut pa = page_pa as usize;
        let mut va = self.base_page;
        for _ in 0..n_phy_page {
            page_insert(pgdir, pa2page(pa), va, ATTRIB_AP_RW_ALL);
            pa += PGSIZE;
            va += PGSIZE;
        }

        let page = unsafe { std::slice::from_raw_parts_mut(page_pa as *mut usize as *mut Page, npage) };
        for i in 0..npage {
            page[i].flags = 0;
            page[i].property = 0;
            page[i].set_page_ref(0);
        }
        page[0].property = npage as u32;
        page[0].SetPageProperty();
        self.n_free += npage as u32;
        //TODO
        self.base_paddr = 0;
        // list_add(&free_list, &(base->page_link));
        kprintln!("init user_memap: {:x} property: {}", self.base_page, page[0].property);
        unsafe { self.free_list.push(self.base_page as *mut usize); }
    }

    pub fn init_memmap(&mut self, base: usize, npage: usize, begin: usize) {
        let page = unsafe { std::slice::from_raw_parts_mut(base as *mut usize as *mut Page, npage) };
        for i in 0..npage {
            page[i].flags = 0;
            page[i].property = 0;
            page[i].set_page_ref(0);
        }
        page[0].property = npage as u32;
        page[0].SetPageProperty();
        self.n_free += npage as u32;
        //TODO
        self.base_page = base;
        self.base_paddr = begin;
        // list_add(&free_list, &(base->page_link));
        kprintln!("init memap: {:x} property: {}", begin, page[0].property);
        unsafe { self.free_list.push(self.base_page as *mut usize); }

        unsafe { 
            // BACKUP_ALLOCATOR = &*(self as *mut Allocator);
            BACKUP_ALLOCATOR.base_paddr = self.base_paddr;
            BACKUP_ALLOCATOR.base_page = self.base_page;
            BACKUP_ALLOCATOR.free_list = self.free_list;
            BACKUP_ALLOCATOR.n_free = self.n_free; 
        }
    }

    pub fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let npage = align_up(layout.size(), PGSIZE) / PGSIZE;
        // kprintln!("try alloc: {} {}", npage, self.n_free);
        if npage as u32 > self.n_free {
            // kprintln!("n_free: {}, npage: {}", self.n_free, npage);
            return Err( AllocErr::Exhausted { request: layout } );
        }
        
        let mut page = None;
        let mut prev = None;
        for i in self.free_list.iter_mut() {
            let mut p = unsafe { &mut *(i.value() as *mut Page) };
            // kprintln!("loop page: {:x} property: {}", p as *mut Page as *mut usize as usize, p.property);
            if p.property >= npage as u32 {
                page = Some(p);
                break;
            }
            prev = Some(p);
        }
        
        match page {
            Some(page) => {
                let mut page_addr = page as *const Page as *mut usize;
                // kprintln!("found page: {:x}, a: {:x}", page_addr as usize, page2kva(page as *const Page));
                if page.property > npage as u32 {
                    let p = unsafe { &mut *((page_addr as usize+ npage * mem::size_of::<Page>()) as *mut Page) };
                    p.property = page.property - npage as u32;
                    p.SetPageProperty();
                    unsafe { page.list_entry.push(p as *const Page as *mut usize) }
                }

                match prev {
                    Some(prev) => unsafe { prev.list_entry.del() },
                    _ => unsafe { self.free_list.remove_head() },
                }

                let mut pages = unsafe { std::slice::from_raw_parts_mut(page as *mut Page, npage) };
                for i in 0..npage {
                    pages[i].SetPageUsed();
                }

                self.n_free -= npage as u32;
                page.property = npage as u32;
                
                // kprintln!("PPN: {:x}", (page as *const Page as *mut usize as usize) - self.base_page);
                // kprintln!("alloc addr: {:x}", offset + self.base_paddr);
                // kprintln!("offset: {:x} base_page: {:x} base_paddr: {:x}", offset, self.base_page, self.base_paddr);
                // kprintln!("alloc pa: {:x} base_page {:x} base_paddr {:x} n_free {}", self.page2addr(page), self.base_page, self.base_paddr, self.n_free);
                return Ok(self.page2addr(page) as *mut usize as * mut u8);
            }
            _ => Err( AllocErr::Exhausted { request: layout } )
        }
    }

    pub fn alloc_at(&mut self, va: usize, layout: Layout, pgdir: *const usize) -> Result<*mut u8, AllocErr> {
        kprintln!("try alloc");

        switch_pgdir(pgdir);

        switch_back();
        kprintln!("get here");
        switch_pgdir(pgdir);
        
        let npage = align_up(layout.size(), PGSIZE) / PGSIZE;
        let addr = align_down(va, PGSIZE);

        switch_back();
        kprintln!("try alloc_at: {} n_free: {}", va, self.n_free);
        switch_pgdir(pgdir);
        if npage as u32 > self.n_free {
            switch_back();
            kprintln!("n_free: {}", self.n_free);
            switch_back();
            return Err( AllocErr::Exhausted { request: layout } );
        }
        
        let mut page = None;
        let mut prev = None;
        
        let mut tempA = Allocator::new();
        tempA.base_paddr = self.base_paddr;
        tempA.base_page = self.base_page;

        for i in self.free_list.iter_mut() {
            // kprintln!("enter loop");
            let mut p = i.value();
            let phy_page = unsafe { &mut *(p as *mut Page) };
            switch_back();
            kprintln!("loop page: va: {:x}, property {}", tempA.page2addr(p as *mut Page), phy_page.property);
            kprintln!("[ {:x}, {:x} ]", addr, addr + npage * PGSIZE);
            kprintln!("[ {:x}, {:x} ]", tempA.page2addr(p as *mut Page), tempA.page2addr(p as *mut Page) + (phy_page.property as usize) * PGSIZE);
            switch_pgdir(pgdir);
            if addr >= tempA.page2addr(p as *mut Page) && addr + npage * PGSIZE <= tempA.page2addr(p as *mut Page) + (phy_page.property as usize) * PGSIZE {
                page = Some(phy_page);
                break;
            }
            prev = Some(phy_page);
        }

        

        match page {
            Some(page) => {
                let prev_npage = ((addr - self.page2addr(page as *mut Page)) / PGSIZE) as usize;
                let next_npage = page.property as usize - npage - prev_npage as usize;
                kprintln!("prev_npage: {}, next_npage: {}", prev_npage, next_npage);
                let mut page_addr = page as *const Page;
                let alloc_page = unsafe { page_addr.add(prev_npage) };

                if next_npage > 0 {
                    // let next_page_va = page_addr as usize+ (prev_npage+npage) * mem::size_of::<Page>();
                    // let next_page = unsafe { &mut *(next_page_va as *mut Page) };
                    let mut next_page = unsafe { &mut *(page_addr.add(prev_npage + npage) as *mut Page) };
                    next_page.SetPageProperty();
                    next_page.property = next_npage as u32;
                    unsafe { page.list_entry.push(next_page as *mut Page as *mut usize) }
                }

                if prev_npage > 0 {
                    page.property = prev_npage as u32;
                } else {
                    match prev {
                        Some(prev) => unsafe { 
                            prev.list_entry.del() 
                        },
                        _ => unsafe { self.free_list.remove_head() },
                    }
                }

                let mut pages = unsafe { std::slice::from_raw_parts_mut(page as *mut Page, npage) };
                for i in 0..npage {
                    pages[i].SetPageUsed();
                }
                
                self.n_free -= npage as u32;
                
                switch_back();
                kprintln!("alloc addr at: {:x}", self.page2addr(alloc_page) as *mut usize as usize);
                // kprintln!("offset: {:x} base_page: {:x} base_paddr: {:x}", offset, self.base_page, self.base_paddr);
                switch_back();
                return Ok(self.page2addr(alloc_page) as *mut usize as * mut u8);
            }
            _ => { 
                switch_back();
                Err( AllocErr::Exhausted { request: layout } )
            }
        }

    }

    pub fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        // nothing
        // kprintln!("dealloc {:x} page: {:x}", _ptr as *mut usize as usize, pa2page(_ptr as *mut usize as usize) as *mut usize as usize);
        let npage = align_up(_layout.size(), PGSIZE) / PGSIZE;

        let pages = unsafe { std::slice::from_raw_parts_mut(KERNEL_PAGES as *mut Page, NPAGE) };
        let mut base_page_addr: usize = 0;

        for i in 0..npage {
            //assert(!PageReserved(p) && !PageProperty(p));
            if i == 0 {
                pages[PPN(_ptr as usize) + i].property = npage as u32;
                pages[PPN(_ptr as usize) + i].SetPageProperty();
                base_page_addr = &pages[PPN(_ptr as usize) + i] as *const Page as usize;
                // base_page = Some(&pages[PPN(_ptr)])
            }
            pages[PPN(_ptr as usize) + i].flags = 0;
            pages[PPN(_ptr as usize) + i].set_page_ref(0);
        }

        let mut prev = false;
        let mut next = false;
        let base_page = unsafe { &mut *(base_page_addr as *mut Page) };
        let mut next_prev = None;

        let pages = unsafe { std::slice::from_raw_parts_mut(base_page as *mut Page, npage) };
        for i in 0..npage {
            pages[i].ClearPageUsed();
        }
        
        // kprintln!("base_page_addr: {:x}", base_page_addr);
        for i in self.free_list.iter_mut() {
            let mut p = unsafe { &mut *(i.value() as *mut Page) };
            // kprintln!("iter page: {:x}. base: offset = {:x}, property = {}, pagesize = {}", i.value() as usize, mem::size_of::<Page>() * base_page.property as usize, base_page.property, mem::size_of::<Page>());
            if base_page_addr + mem::size_of::<Page>() * base_page.property as usize == i.value() as usize {
                base_page.property += p.property;
                p.ClearPageProperty();
                // kprintln!("found next");
                next = true;
                break;
            }
            next_prev = Some(p);
        }

        if next {
            match next_prev {
                Some(next_prev) => unsafe { next_prev.list_entry.del() },
                _ => unsafe { self.free_list.remove_head() },
            }
        }

        
        for i in self.free_list.iter_mut() {
            let mut p = unsafe { &mut *(i.value() as *mut Page) };
            // kprintln!("iter page: {:x}, offset = {:x}, property = {}, pagesize = {}", i.value() as usize, mem::size_of::<Page>() * p.property as usize, p.property, mem::size_of::<Page>());
            if i.value() as usize + mem::size_of::<Page>() * p.property as usize == base_page_addr {
                p.property += base_page.property;
                base_page.ClearPageProperty();
                // kprintln!("found prev");
                prev = true;
                break;
            }
        }

        if !prev {
            unsafe{ self.free_list.push(base_page_addr as *mut usize) };
        } 

        self.n_free += npage as u32;
        // kprintln!("dealloc ed");
    }
    
    pub fn switch_content(&mut self, alloc_from: &Allocator, alloc_to: &mut Allocator) {
        // let temp = self.copy();
        // if allocator as *mut usize as usize != 0 {
        //     self = *allocator;
        // }
        // *allocator = temp;
        alloc_to.n_free = self.n_free;
        alloc_to.base_page = self.base_page;
        alloc_to.base_paddr = self.base_paddr;
        alloc_to.free_list = self.free_list;
        self.n_free = alloc_from.n_free;
        self.base_page = alloc_from.base_page;
        self.base_paddr = alloc_from.base_paddr;
        self.free_list = alloc_from.free_list;
    }

    pub fn clear_page(&mut self, pgdir: *const usize) {
        kprintln!("clear page {}", (&ALLOCATOR).get_n_free());
        let pte = get_pte(pgdir, self.base_page, false).expect("no pte found.");
        let pages_pa = unsafe{ PTE_ADDR(*pte) };
        // kprintln!("pages_pa: {:x}", pages_pa as usize);
        let npage = self.base_page / PGSIZE;
        let pages = unsafe { std::slice::from_raw_parts_mut(pages_pa as *mut usize as *mut Page, npage) };
        let mut i = 0;
        let mut cleared = 0;
        for page in pages {
            if page.isUsed() {
                let va = self.page2addr((self.base_page + i * mem::size_of::<Page>()) as *const Page);
                kprintln!("va: {:x}", va);
                match get_pte(pgdir, va, false) {
                    Ok(pte) => {
                        kprintln!("pte: {:x} pa: {:x}", unsafe{ *pte }, PTE_ADDR(unsafe{*pte}));
                        if unsafe{ *pte & PTE_V != 0} {
                            cleared += 1;
                            let pa = PTE_ADDR( unsafe{ *pte })as *mut u8;
                            unsafe { (&ALLOCATOR).dealloc(pa, Layout::from_size_align_unchecked(PGSIZE, PGSIZE)); }
                        }
                    },
                    Err(_) => {}
                }
            }
            i += 1; 
        }

        unsafe { (&ALLOCATOR).dealloc(pages_pa as *mut u8, Layout::from_size_align_unchecked(768 * PGSIZE, PGSIZE)); }

        kprintln!("cleared {} n_free {}", cleared, (&ALLOCATOR).get_n_free());
    }

    pub fn get_n_free(&self) -> u32 {
        self.n_free
    }

    fn page2addr(&self, page: *const Page) -> usize {
        let offset = (((page as *mut usize as usize) - self.base_page) / mem::size_of::<Page>()) * PGSIZE;
        let addr = (offset + self.base_paddr) as usize;
        addr
    }

    fn addr2page(&self, addr: usize) -> *mut Page {
        let offset = ((addr - self.base_paddr) / PGSIZE * mem::size_of::<Page>()) as usize ;
        let page = (self.base_page + offset) as *mut usize as *mut Page;
        page
    }

    pub fn copy_page(&mut self, src_pgdir: *const usize, dst_pgdir: *const usize) {
        kprintln!("copy page");
        let pte = get_pte(src_pgdir, self.base_page, false).expect("no pte found.");
        let pte_dst = get_pte(dst_pgdir, self.base_page, false).expect("no pte found.");
        let pages_pa = unsafe{ PTE_ADDR(*pte) };
        let pages_pa_dst = unsafe{ PTE_ADDR(*pte_dst) };
        // kprintln!("pages_pa: {:x}", pages_pa as usize);
        let npage = self.base_page / PGSIZE;
        let pages = unsafe { std::slice::from_raw_parts_mut(pages_pa as *mut usize as *mut Page, npage) };

        memcpy(pages_pa_dst as *mut u8, pages_pa as *mut u8, npage * mem::size_of::<Page>());
        
        let mut i = 0;
        for page in pages {
            // kprintln!("copy {} {:?} {:x}", i, page.isUsed(), page as *mut Page as *mut usize as usize);
            if page.isUsed() {
                let va = self.page2addr((self.base_page + i * mem::size_of::<Page>()) as *const Page);
                match get_pte(src_pgdir, va, false) {
                    Ok(pte) => {
                        if unsafe{ *pte & PTE_V != 0} {
                            let src_pa = PTE_ADDR( unsafe{ *pte }) as *mut u8;
                            let PXN = 0x1 << 53 as usize;
                            let UXN = 0x0 << 54 as usize;
                            let perm = UXN | PXN | ATTRIB_AP_RW_ALL;
                            let dst_pa = pgdir_alloc_page(dst_pgdir, va, perm).expect("user alloc page failed");
                            kprintln!("src_pa: {:x}, dst_pa: {:x}", src_pa as usize , dst_pa as usize);
                            memcpy(dst_pa as *mut u8, src_pa as *mut u8, PGSIZE);
                        }
                    },
                    Err(_) => {}
                }
            }
            i += 1;
        }
    }
    
}

pub fn alloc_page_at(allocator: &mut Allocator, va: usize, pgdir: *const usize) -> Result<*mut u8, AllocErr> {
    unsafe { allocator.alloc_at(va, Layout::from_size_align_unchecked(PGSIZE, PGSIZE), pgdir) }
}