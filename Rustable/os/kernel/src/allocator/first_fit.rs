use alloc::heap::{AllocErr, Layout};
use std;
use std::mem;
use allocator::util::*;
use allocator::linked_list::LinkedList;

/// A "bump" allocator: allocates memory by bumping a pointer; never frees.
#[derive(Debug)]
pub struct Allocator {
    free_list: LinkedList,
    n_free: u32,
}

impl Allocator {
    pub fn new() -> Allocator {
        Self {
            free_list: LinkedList::new(),
            n_free: 0
        }
    }

    pub fn init_memmap(&mut self, base: usize, npage: usize) {
        let mut page = unsafe { std::slice::from_raw_parts_mut(base, npage) };
        for i in 0..npage {
            page[i].flags = 0;
            page[i].property = 0;
            page[i].set_page_ref(0);
        }
        page[0].property = npage;
        SetPageProperty(page[0]);
        self.n_free += napge;
        //TODO
        let base_addr = &base as *const usize;
        // list_add(&free_list, &(base->page_link));
        free_list.push(base_addr);
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
    pub fn alloc(&mut self, npage: u32) -> Result<*mut Page, AllocErr> {
        if (npage > self.n_free) {
            return Err( AllocErr::Exhausted { request: PGSIZE } )
        }
        
        // TODO
        let page = None;
        let prev: Page;
        for i in self.free_list.iter_mut() {
            // #[cfg(test)]
            // println!( "popping off alloc_start: {:#?}", alloc_start);
            let p = unsafe { &*(i.value()) };
            if p.property >= npage {
                page = Some(p);
                break;
            }
            prev = p;
        }
        
        match page {
            Some(page) => {
                if page.property > npage {
                    let page_addr = page as *mut Page as *mut usize;
                    let p = unsafe { &*(page_addr.add(n * mem::size_of::<Page>() as *mut Page)) };
                    p.property = page.property - n;
                    p.SetPageProperty();
                    page.list_entry.push_after(p as *mut Page as *mut usize)
                }
                prev.list_entry.del();
                self.n_free -= n;
                page.ClearPageProperty();
                
                return Ok(page)
            }
        }
        Err( AllocErr::Exhausted { request: PGSIZE } )
        
    }

    pub fn dealloc(&mut self, base: usize, npage: u32) -> Result<*mut Page, AllocErr> {
        // nothing
    }
}
