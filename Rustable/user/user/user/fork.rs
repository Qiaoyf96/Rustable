#![feature(asm, lang_items)]
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! { loop{} }

mod syscall;
use syscall::*;

#[no_mangle]
pub extern "C" fn kmain() {
    for i in 0..10 {
        let pid = sys_fork();
        if pid == 0 {
            sys_print(i);
            sys_exit();
        }
        assert!(pid > 0);
    }
    sys_exit();
}