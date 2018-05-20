use alloc::heap::{AllocErr, Layout};
use std;
use std::mem;
use allocator::util::*;
use allocator::linked_list::LinkedList;
use allocator::page::{PGSIZE, Page, PPN, KERNEL_PAGES, NPAGE, MAXPA, page2va, page2kva, pa2page, ATTRIB_AP_RW_ALL, page2pa};
use allocator::alloc_pages;
use mm::pmm::page_insert;

use console::kprintln;

pub static mut BACKUP_ALLOCATOR : &Allocator = &Allocator {
    base_paddr: 0,
    base_page: 0,
    free_list: LinkedList::new(),
    n_free: 0,
};

pub static mut USER_ALLOCATOR : &Allocator = &Allocator {
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
    base_paddr: usize,
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
            Err(_) => { return; }
        };
        
        kprintln!("physical page addr: {:x}", page_pa as usize);        

        let mut pa = page_pa as usize;
        let mut va = self.base_page;
        for i in 0..n_phy_page {
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
            BACKUP_ALLOCATOR = &*(self as *mut Allocator);
            // BACKUP_ALLOCATOR.base_paddr = self.base_paddr;
            // BACKUP_ALLOCATOR.base_page = self.base_page;
            // BACKUP_ALLOCATOR.free_list = self.free_list;
            // BACKUP_ALLOCATOR.n_free = self.n_free; 
        }
    }

    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted
    /// (`AllocError::Exhausted`) or `layout` does not meet this allocator's
    /// size or alignment constraints (`AllocError::Unsupported`).
    pub fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let npage = align_up(layout.size(), PGSIZE) / PGSIZE;
        kprintln!("try alloc: {} {}", npage, self.n_free);
        if npage as u32 > self.n_free {
            kprintln!("n_free: {}, npage: {}", self.n_free, npage);
            return Err( AllocErr::Exhausted { request: layout } );
        }
        
        let mut page = None;
        let mut prev = None;
        for i in self.free_list.iter_mut() {
            let mut p = unsafe { &mut *(i.value() as *mut Page) };
            kprintln!("loop page: {:x} property: {}", p as *mut Page as *mut usize as usize, p.property);
            if p.property >= npage as u32 {
                page = Some(p);
                break;
            }
            prev = Some(p);
        }
        
        match page {
            Some(page) => {
                let mut page_addr = page as *const Page as *mut usize;
                kprintln!("found page: {:x}, pa: {:x}", page_addr as usize, page2kva(page as *const Page));
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

                self.n_free -= npage as u32;
                page.ClearPageProperty();
                
                let offset = (((page as *const Page as *mut usize as usize) - self.base_page) / mem::size_of::<Page>()) * PGSIZE;
                kprintln!("PPN: {:x}", (page as *const Page as *mut usize as usize) - self.base_page);
                kprintln!("alloc addr: {:x}", offset + self.base_paddr);
                kprintln!("offset: {:x} base_page: {:x} base_paddr: {:x}", offset, self.base_page, self.base_paddr);
                
                return Ok((offset + self.base_paddr) as *mut usize as * mut u8);
            }
            _ => Err( AllocErr::Exhausted { request: layout } )
        }
    }

    pub fn alloc_at(&mut self, va: usize, layout: Layout) -> Result<*mut u8, AllocErr> {
        let npage = align_up(layout.size(), PGSIZE) / PGSIZE;
        let addr = align_down(va, PGSIZE);
        kprintln!("try alloc: {}", npage);
        if npage as u32 > self.n_free {
            return Err( AllocErr::Exhausted { request: layout } );
        }
        
        let mut page = None;
        let mut prev = None;
        for i in self.free_list.iter_mut() {
            let mut p = unsafe { &mut *(i.value() as *mut Page) };
            if addr >= page2va(p) && addr < page2va(p) + (p.property as usize) * PGSIZE && p.property >= npage as u32 {
                page = Some(p);
                break;
            }
            prev = Some(p);
        }

        match page {
            Some(page) => {
                let prev_npage = ((addr - page2va(page)) / PGSIZE) as usize;
                let next_npage = page.property as usize - npage - prev_npage as usize;
                let mut page_addr = page as *const Page as *mut usize;
                let alloc_page = unsafe { &mut *((page_addr as usize+ npage * mem::size_of::<Page>()) as *mut Page) };

                if next_npage > 0 {
                    let next_page = unsafe { &mut *((page_addr as usize+ (prev_npage+npage) * mem::size_of::<Page>()) as *mut Page) };
                    next_page.SetPageProperty();
                    unsafe { page.list_entry.push(next_page as *const Page as *mut usize) }
                }

                if prev_npage > 0 {
                    page.property = prev_npage as u32;
                } else {
                    match prev {
                        Some(prev) => unsafe { prev.list_entry.del() },
                        _ => unsafe { self.free_list.remove_head() },
                    }
                }
                
                self.n_free -= npage as u32;
                
                let offset = (((alloc_page as *const Page as *mut usize as usize) - self.base_page) / mem::size_of::<Page>()) * PGSIZE;
                kprintln!("alloc addr: {:x}", offset + self.base_paddr);
                // kprintln!("offset: {:x} base_page: {:x} base_paddr: {:x}", offset, self.base_page, self.base_paddr);
                return Ok((offset + self.base_paddr) as *mut usize as * mut u8);
            }
            _ => Err( AllocErr::Exhausted { request: layout } )
        }
    }

    pub fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        // nothing
        kprintln!("dealloc {:x} page: {:x}", _ptr as *mut usize as usize, pa2page(_ptr as *mut usize as usize) as *mut usize as usize);
        let npage = align_up(_layout.size(), PGSIZE) / PGSIZE;

        let mut pages = unsafe { std::slice::from_raw_parts_mut(KERNEL_PAGES as *mut Page, NPAGE) };
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
        let mut base_page = unsafe { &mut *(base_page_addr as *mut Page) };
        let mut next_prev = None;
        
        kprintln!("base_page_addr: {:x}", base_page_addr);
        for i in self.free_list.iter_mut() {
            let mut p = unsafe { &mut *(i.value() as *mut Page) };
            kprintln!("iter page: {:x}. base: offset = {:x}, property = {}, pagesize = {}", i.value() as usize, mem::size_of::<Page>() * base_page.property as usize, base_page.property, mem::size_of::<Page>());
            if base_page_addr + mem::size_of::<Page>() * base_page.property as usize == i.value() as usize {
                base_page.property += p.property;
                p.ClearPageProperty();
                kprintln!("found next");
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
            kprintln!("iter page: {:x}, offset = {:x}, property = {}, pagesize = {}", i.value() as usize, mem::size_of::<Page>() * p.property as usize, p.property, mem::size_of::<Page>());
            if i.value() as usize + mem::size_of::<Page>() * p.property as usize == base_page_addr {
                p.property += base_page.property;
                base_page.ClearPageProperty();
                kprintln!("found prev");
                prev = true;
                break;
            }
        }

        if !prev {
            unsafe{ self.free_list.push(base_page_addr as *mut usize) };
        } 
        self.n_free += npage as u32;
        kprintln!("dealloc ed");
    }
    
    pub fn switch_content(&mut self, alloc_from: *mut Allocator, alloc_to: *mut Allocator) {
        // let temp = self.copy();
        // if allocator as *mut usize as usize != 0 {
        //     self = *allocator;
        // }
        // *allocator = temp;
        unsafe {
            (*alloc_to).n_free = self.n_free;
            (*alloc_to).base_page = self.base_page;
            (*alloc_to).base_paddr = self.base_paddr;
            (*alloc_to).free_list = self.free_list;
            self.n_free = (*alloc_from).n_free;
            self.base_page = (*alloc_from).base_page;
            self.base_paddr = (*alloc_from).base_paddr;
            self.free_list = (*alloc_from).free_list;
        }
    }
}

pub fn alloc_page_at(allocator: &mut Allocator, va: usize) -> Result<*mut u8, AllocErr> {
    unsafe { allocator.alloc_at(va, Layout::from_size_align_unchecked(PGSIZE, PGSIZE)) }
}