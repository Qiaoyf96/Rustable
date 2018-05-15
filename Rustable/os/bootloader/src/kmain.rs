#![feature(asm, lang_items)]

extern crate xmodem;
extern crate pi;

use std::fmt::Write;

pub mod lang_items;
pub mod mm_init;

/// Start address of the binary to load and of the bootloader. 0xffffff0001000000
const BINARY_START_ADDR: usize = 0xffffff0000800000;
const BOOTLOADER_START_ADDR: usize = 0xffffff0004000000;

/// Pointer to where the loaded binary expects to be laoded.
const BINARY_START: *mut u8 = BINARY_START_ADDR as *mut u8;

/// Free space between the bootloader and the loaded binary's start address.
const MAX_BINARY_SIZE: usize = BOOTLOADER_START_ADDR - BINARY_START_ADDR;

/// Branches to the address `addr` unconditionally.
fn jump_to(addr: *mut u8) -> ! {
    unsafe {
        asm!("br $0" : : "r"(addr as usize));
        loop { asm!("nop" :::: "volatile")  }
    }
}

#[no_mangle]
pub extern "C" fn kmain() {
    // FIXME: Implement the bootloader.
    let mut uart = pi::uart::MiniUart::new();
    uart.set_read_timeout(750);
    loop {
        let mut addr = unsafe { std::slice::from_raw_parts_mut(BINARY_START, MAX_BINARY_SIZE) };
        // addr[0] = 0x1;
        match xmodem::Xmodem::receive(&mut uart, std::io::Cursor::new(addr)) {
            Ok(_) => { jump_to(BINARY_START); },
            Err(err) => match err.kind() {
                std::io::ErrorKind::TimedOut => continue,
                std::io::ErrorKind::InvalidData => continue,
                _ => uart.write_fmt(format_args!("Error: {:?}\r\n", err)).unwrap(),
            },
        }
    }
}
