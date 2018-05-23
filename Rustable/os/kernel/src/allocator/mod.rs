mod linked_list;
use ALLOCATOR;
pub mod page;
pub mod util;

#[path = "first_fit.rs"]
pub mod imp;

#[cfg(test)]
mod tests;

pub use self::page::{Page, PGSIZE, UXN, PXN, ATTRIB_AP_RW_ALL, PTE_ADDR, PTE_V};

use mutex::Mutex;
use alloc::heap::{Alloc, AllocErr, Layout};
use std::cmp::max;

use pi::atags::Atags;

use process::process::utils::memset;

/// Thread-safe (locking) wrapper around a particular memory allocator.
// #[derive(Debug)]
pub struct Allocator(Mutex<Option<imp::Allocator>>);

impl Allocator {
    /// Returns an uninitialized `Allocator`.
    ///
    /// The allocator must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        Allocator(Mutex::new(None))
    }

    /// Initializes the memory allocator.
    ///
    /// # Panics
    ///
    /// Panics if the system's memory map could not be retrieved.
    pub fn initialize(&self) {
        // let (start, end) = memory_map().expect("failed to find memory map");
        *self.0.lock() = Some(imp::Allocator::new());
    }

    pub fn init_memmap(&self, base: usize, npage: usize, begin: usize) {
        self.0.lock().as_mut().expect("allocator uninitialized").init_memmap(base, npage, begin);
    }

    pub fn alloc_at(&self, addr: usize, layout: Layout, pgdir: *const usize) -> Result<*mut u8, AllocErr> {
        self.0.lock().as_mut().expect("allocator uninitialized").alloc_at(addr, layout, pgdir)
    }

    pub fn switch_content(&self, alloc_from: &imp::Allocator, alloc_to: &mut imp::Allocator) {
        // kprintln!("SWITCH");
        // if allocator as *const usize as usize == 0 {
        //     kprintln!("SWITCH");
        //     return self.0.lock().as_mut().expect("allocator uninitialized") as *const imp::Allocator;
        // }
        // let mut backup = self.0.lock();
        // unsafe { *self.0.lock() = Some(ptr::read(allocator)); }
        // backup.as_mut().expect("allocator uninitialized") as *const imp::Allocator
        self.0.lock().as_mut().expect("allocator uninitialized").switch_content(alloc_from, alloc_to);
    }

    

}

unsafe impl<'a> Alloc for &'a Allocator {

    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        self.0.lock().as_mut().expect("allocator uninitialized").alloc(layout)
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        self.0.lock().as_mut().expect("allocator uninitialized").dealloc(ptr, layout);
    }
}

extern "C" {
    static _end: u8;
}

pub fn alloc_page() -> Result<*mut u8, AllocErr> {
    let pa = unsafe { (&ALLOCATOR).alloc(Layout::from_size_align_unchecked(PGSIZE, PGSIZE)).expect("alloc page failed") };
    unsafe { memset(pa as *mut u8, 0, PGSIZE); };
    Ok(pa)
}

pub fn dealloc_page(ptr: *mut u8) {
    dealloc_pages(ptr, 1);
}

pub fn alloc_pages(npage: usize) -> Result<*mut u8, AllocErr> {
    unsafe { (&ALLOCATOR).alloc(Layout::from_size_align_unchecked(npage * PGSIZE, PGSIZE)) }

}

pub fn dealloc_pages(ptr: *mut u8, npage: usize) {
    unsafe { (&ALLOCATOR).dealloc(ptr, Layout::from_size_align_unchecked(npage * PGSIZE, PGSIZE)); }
}



