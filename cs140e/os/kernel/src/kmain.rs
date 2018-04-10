#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(never_type)]
#![feature(ptr_internals)]

extern crate pi;
extern crate stack_vec;

pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;

use pi::timer::{spin_sleep_ms, current_time};



#[no_mangle]
pub extern "C" fn kmain() {
    // FIXME: Start the shell.
    spin_sleep_ms(1000);
    // console::kprint!("{}\n", begin);
    shell::shell("Rainable: ");
}
