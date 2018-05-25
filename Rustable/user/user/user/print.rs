#![feature(asm, lang_items)]
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! { loop{} }

#[no_mangle]
pub extern "C" fn kmain() {
    for i in 0..1000 {
        sys_print(i);
    }
    sys_exit();
}
