#[inline(always)]
pub fn tlb_invalidate(va: usize) {
    unsafe{
        asm!("dsb ishst
              tlbi vmalle1is
              dsb ish
              tlbi vmalle1is
              isb");
    }
}

/// Returns the current stack pointer.
#[inline(always)]
pub fn sp() -> *const u8 {
    let ptr: usize;
    unsafe {
        asm!("mov $0, sp" : "=r"(ptr));
    }

    ptr as *const u8
}

/// Returns the current exception level.
///
/// # Safety
/// This function should only be called when EL is >= 1.
#[inline(always)]
pub unsafe fn current_el() -> u8 {
    let el_reg: u64;
    asm!("mrs $0, CurrentEL" : "=r"(el_reg));
    ((el_reg & 0b1100) >> 2) as u8
}

#[inline(always)]
pub unsafe fn get_far() -> usize {
    let far: usize;
    asm!("mrs $0, far_el1" : "=r"(far));
    far
}

#[inline(always)]
pub unsafe fn get_ttbr0() -> usize {
    let ttbr0: usize;
    asm!("mrs $0, ttbr0_el1" : "=r"(ttbr0));
    ttbr0
}

/// Returns the SPSel value.
#[inline(always)]
pub fn sp_sel() -> u8 {
    let ptr: u32;
    unsafe {
        asm!("mrs $0, SPSel" : "=r"(ptr));
    }

    (ptr & 1) as u8
}

/// Returns the core currently executing.
///
/// # Safety
///
/// This function should only be called when EL is >= 1.
pub unsafe fn affinity() -> usize {
    let x: usize;
    asm!("mrs     $0, mpidr_el1
          and     $0, $0, #3"
          : "=r"(x));

    x
}

/// A NOOP that won't be optimized out.
pub fn nop() {
    unsafe {
        asm!("nop" :::: "volatile");
    }
}

pub fn wfi() {
    unsafe {
        asm!("wfi" :::: "volatile");
    }
}
