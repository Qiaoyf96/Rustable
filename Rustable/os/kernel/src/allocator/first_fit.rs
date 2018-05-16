use alloc::heap::{AllocErr, Layout};
use std;
use std::mem;
use allocator::util::*;
use allocator::linked_list::LinkedList;
use allocator::page::{PGSIZE, Page, PPN, VPN};

use console::kprintln;

/// A "bump" allocator: allocates memory by bumping a pointer; never frees.
#[derive(Debug)]
pub struct Allocator {
    free_list: LinkedList,
    n_free: u32,
    base_addr: usize,
    base_phy_addr: usize,
    page_list_addr: usize,
    page_list_size: usize,
}

impl Allocator {
    pub fn new() -> Allocator {
        Self {
            base_phy_addr: 0,
            base_addr: 0,
            free_list: LinkedList::new(),
            n_free: 0,
            page_list_addr: 0,
            page_list_size: 0,
        }
    }

    pub fn init_page_list(&mut self, page_list_addr: usize, page_list_size: usize) {
        self.page_list_addr = page_list_addr;
        self.page_list_size = page_list_size;
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
        self.base_addr = base;
        self.base_phy_addr = begin;
        // list_add(&free_list, &(base->page_link));
        kprintln!("init memap: {:x} property: {}", begin, page[0].property);
        unsafe { self.free_list.push(self.base_addr as *mut usize); }
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
        // kprintln!("layout size: {} pagesize: {}", layout.size(), PGSIZE);
        let npage = align_up(layout.size(), PGSIZE) / PGSIZE;
        kprintln!("try alloc: {}", npage);
        if npage as u32 > self.n_free {
            return Err( AllocErr::Exhausted { request: layout } );
        }
        
        // TODO
        let mut page = None;
        let mut prev = None;
        for i in self.free_list.iter_mut() {
            // #[cfg(test)]
            // println!( "popping off alloc_start: {:#?}", alloc_start);
            let mut p = unsafe { &mut *(i.value() as *mut Page) };
            if p.property >= npage as u32 {
                page = Some(p);
                // kprintln!("found!");
                break;
            }
            prev = Some(p);
        }
        
        match page {
            Some(page) => {
                if page.property > npage as u32 {
                    let mut page_addr = page as *const Page as *mut usize;
                    // kprintln!("find page addr: {:x}", page_addr as usize);
                    let p = unsafe { &mut *((page_addr.add(npage * mem::size_of::<Page>())) as *mut Page) };
                    p.property = page.property - npage as u32;
                    p.SetPageProperty();
                    // kprintln!("split {:x} {:x}", page.list_entry.head as usize, p as *const Page as *mut usize as usize);
                    unsafe { page.list_entry.push(p as *const Page as *mut usize) }
                    
                }
                // kprintln!("freelist before: {:x}", self.free_list.head as usize);
                match prev {
                    Some(prev) => unsafe { prev.list_entry.del() },
                    _ => unsafe { self.free_list.remove_head() },
                }
                // kprintln!("freelist after: {:x}", self.free_list.head as usize);
                self.n_free -= npage as u32;
                page.ClearPageProperty();
                
                let offset = (((page as *const Page as *mut usize as usize) - self.base_addr) / mem::size_of::<Page>()) * PGSIZE;
                kprintln!("alloc addr: {:x}", offset + self.base_phy_addr);
                // kprintln!("offset: {:x} base_phy_addr: {:x}", offset, self.base_phy_addr);
                return Ok((offset + self.base_phy_addr) as *mut usize as * mut u8);
            }
            _ => Err( AllocErr::Exhausted { request: layout } )
        }
    }

    pub fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        // nothing
        kprintln!("dealloc {:x}", _ptr as *mut usize as usize);
        let npage = align_up(_layout.size(), PGSIZE) / PGSIZE;

        let mut page_list = unsafe { std::slice::from_raw_parts_mut(self.page_list_addr as *mut Page, self.page_list_size) };
        let mut base_page_addr: usize = 0;

        for i in 0..npage {
            //assert(!PageReserved(p) && !PageProperty(p));
            if i == 0 {
                page_list[PPN(_ptr as usize) + i].property = npage as u32;
                page_list[PPN(_ptr as usize) + i].SetPageProperty();
                base_page_addr = &page_list[PPN(_ptr as usize) + i] as *const Page as usize;
                // base_page = Some(&page_list[PPN(_ptr)])
            }
            page_list[PPN(_ptr as usize) + i].flags = 0;
            page_list[PPN(_ptr as usize) + i].set_page_ref(0);
        }

        let mut prev = false;
        let mut next = false;
        let mut base_page = unsafe { &mut *(base_page_addr as *mut Page) };
        let mut next_prev = None;
        
        for i in self.free_list.iter_mut() {
            let mut p = unsafe { &mut *(i.value() as *mut Page) };
            if base_page_addr + mem::size_of::<Page>() * base_page.property as usize == i.value() as usize {
                base_page.property += p.property;
                p.ClearPageProperty();
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
            if i.value() as usize + mem::size_of::<Page>() * p.property as usize == base_page_addr {
                p.property += base_page.property;
                base_page.ClearPageProperty();
                prev = true;
            }
        }

        if !prev {
            unsafe{ self.free_list.push(base_page_addr as *mut usize) };
        } 

        // for i in self.free_list.iter_mut() {
        //     let mut p = unsafe { &mut *(i.value() as *mut Page) };
        //     base_page = base_addr as *mut Page;
        //     if base_addr + mem::size_of::<Page>() * base_page.property == i.value() {  // base + base->property == p
        //         base_page.property += p.property;
        //         p.ClearPageProperty();
        //         p_addr = i.value();
        //         next = Some(p);
        //         break;
        //     } else if i.value() + mem::size_of::<Page>() * p.property == base_addr {
        //         p.property += base_page.property;
        //         base_page.ClearPageProperty();
        //         base_addr = i.value();
        //         p_addr = i.value();
        //         prev = Some(p);
        //     }
        //     if p_addr > base_addr {
        //         break;
        //     }

        // }

        // if next != None {
        //     if prev == None{
        //         unsafe { next.list_entry.push(base_addr as *const Page as *mut usize) }
        //     }
        //     unsafe { next.list_entry.del() }
        // }
        // if prev == None && next == None {
        //     if (p_addr > base_addr && p_addr != base_addr + npage) {
        //         //list_add_before(&(p->page_link), &(base->page_link));
        //     } else {
        //         //list_add_before(&free_list, &(base->page_link));
        //     }
        // }

        // unsafe { self.free_list.push(&page_list[PPN(_ptr as usize)] as *const Page as *mut usize) };
        self.n_free += npage as u32;
        kprintln!("dealloc ed");
    }
}
