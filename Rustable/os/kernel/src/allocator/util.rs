use mm::vm::get_pte;
use allocator::page::{PTE_ADDR, PADDR, OFFSET};

/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if align == 0 || align & (align - 1) > 0 { panic!("ERROR: Align is not power of 2"); }
    addr / align * align
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_up(addr: usize, align: usize) -> usize {
    if align == 0 || align & (align - 1) > 0 { panic!("ERROR: Align is not power of 2"); }
    (addr + align - 1) / align * align
}

pub fn va2pa(va: usize, pgdir: *const usize) -> usize {
    let pte = get_pte(pgdir, va, false).expect("no pa found");
    PTE_ADDR(unsafe { *pte }) + OFFSET(va)
}

pub fn switch_pgdir(pgdir: *const usize) {
    unsafe {
        asm!("mov x1, $0
            msr ttbr0_el1, x1"
            ::"r"(PADDR(pgdir as usize))::"volatile");
    }; 

    use aarch64::tlb_invalidate;
    tlb_invalidate();
}

pub fn switch_back() {
    switch_pgdir(0x1000000 as *const usize);
}