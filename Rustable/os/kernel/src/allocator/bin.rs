use std::fmt;
use alloc::heap::{AllocErr, Layout};

use allocator::util::*;
use allocator::linked_list::LinkedList;

use std::cmp::min;
use std::cmp::max;
use std::mem::size_of;

const MIN_POWER: usize = 3usize;
const MAX_POWER: usize = 32usize;
const LIST_SIZE: usize = MAX_POWER - MIN_POWER + 1;

/// A simple allocator that allocates based on size classes.
#[derive(Debug)]
pub struct Allocator {
    // FIXME: Add the necessary fields.
    start: usize,
    bins: [LinkedList; LIST_SIZE],
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        if end - start < 8 { panic!("Initial memory is too small"); }
        let mut size: usize = ((end - start) as usize).next_power_of_two();
        if size > end - start { size >>= 1; }
        let power: usize = size.trailing_zeros() as usize;
        if power > MAX_POWER { panic!("Initial memory is too large"); }
        let mut bins = [LinkedList::new(); LIST_SIZE];
        unsafe { bins[power - MIN_POWER].push(start as *mut usize) };
        Allocator { start, bins }
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
        let size = align_up(layout.size(), layout.align()).next_power_of_two() as usize;
        let power = size.trailing_zeros() as usize;
        if power > MAX_POWER { return Err(AllocErr::Exhausted { request: layout }); }
        let mut _power = power;
        while true {
            match self.bins[_power - MIN_POWER].pop() {
                Some(addr) => { // Find an avaliable block
                    if _power <= power { return Ok(addr as *mut u8); }
                    _power -= 1;
                    unsafe {
                        self.bins[_power - MIN_POWER].push(addr.add((1 << _power) / size_of::<usize>()));
                        self.bins[_power - MIN_POWER].push(addr);
                    }
                },
                None => { 
                    _power += 1;
                    if _power > MAX_POWER { return Err(AllocErr::Exhausted { request: layout }); }
                }
            }
        }
        unreachable!();
        // Err(AllocErr::Unsupported { details: "Why error? Impossible?" })
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let mut size = align_up(layout.size(), layout.align()).next_power_of_two() as usize;
        let mut power = size.trailing_zeros() as usize;
        let mut addr = ptr as usize;
        while power < MAX_POWER {
            match self.find_and_pop_buddy(addr, power, size) {
                Some(min_addr) => {
                    power += 1;
                    size <<= 1;
                    addr = min_addr;
                    // unsafe { self.bins[power - MIN_POWER].push(addr as *mut usize); }
                },
                None => {
                    unsafe { self.bins[power - MIN_POWER].push(addr as *mut usize) };
                    return;
                }
            }
        }
    }

    fn find_and_pop_buddy(&mut self, addr: usize, power: usize, size: usize) -> Option<usize> {
        let buddy_addr = ((1 << power) ^ (addr - self.start)) + self.start;
        for node in self.bins[power - MIN_POWER].iter_mut() {
            if node.value() as usize == buddy_addr { // Find buddy
                node.pop();
                return Some(min(buddy_addr, addr));
            }
        }
        None
    }
}
