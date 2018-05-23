#![feature(asm)]

#[no_mangle]
pub extern "C" fn kmain() {
    unsafe { asm!("svc 3" :::: "volatile"); }
}
