
#![crate_type = "staticlib"]
#![feature(asm)]
#![feature(lang_items)]
#![no_std]
#![no_main]
#[cfg(any(target_arch = "x86"))]

mod elf;
mod asm;

use core::slice::from_raw_parts;
// use core::mem::transmute;
use elf::{Elfhdr, Proghdr};
use asm::{inb, insl, outb, outw, jump_to};


const ELF_MAGIC: u32 = 0x464C457F;
const SECTSIZE: u32 = 512;
const elf_addr: usize = 0x10000;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(_msg: core::fmt::Arguments, 
    _file: &'static str, _line: u32, _col: u32) -> !
{
    loop {}
}

// #[stable(feature = "rust1", since = "1.0.0")]
// pub fn transmute<T, U>(e: T) -> U;

// #[inline]
// #[stable(feature = "rust1", since = "1.0.0")]
// pub unsafe fn from_raw_parts<'a, T>(p: *const T, len: usize) -> &'a [T] {
//     transmute(Repr { data: p, len: len })
// }

fn waitdisk() {
	while (inb(0x1F7) & 0xC0) != 0x40 {
	}
}

fn readsect(dst: u32, secno: u32) {
	waitdisk();

	outb(0x1F2, 1);                         // count = 1
	outb(0x1F3, (secno & 0xFF) as u8);
	outb(0x1F4, ((secno >> 8) & 0xFF) as u8);
	outb(0x1F5, ((secno >> 16) & 0xFF) as u8);
	outb(0x1F6, (((secno >> 24) & 0xF) | 0xE0) as u8);
	outb(0x1F7, 0x20 as u8);                      // cmd 0x20 - read sectors

	// wait for disk to be ready
	waitdisk();

	// read a sector
	insl(0x1F0, dst, (SECTSIZE / 4) as i32);
}

fn readseg(_va: u32, count: u32, offset: u32) {
    let end_va = (_va + count) as u32;

    // round down to sector boundary
    let mut va = (_va - offset % SECTSIZE) as u32;

    // translate from bytes to sectors; kernel starts at sector 1
    let mut secno = (offset / SECTSIZE + 1) as u32;

    // If this is too slow, we could read lots of sectors at a time.
    // We'd write more to memory than asked, but it doesn't matter --
    // we load in increasing order.

	while va < end_va {
		readsect(va, secno);
		va += SECTSIZE;
		secno += 1;
	}
}

#[no_mangle]
pub extern fn bootmain() {

	readseg(elf_addr as u32, SECTSIZE * 8, 0);

	let ELFHDRS: &[Elfhdr] = unsafe { from_raw_parts(elf_addr as * const usize as * const Elfhdr, 1) };
	let ELFHDR = ELFHDRS[0].clone();

    if ELFHDR.e_magic == ELF_MAGIC {
		let ph_addr = elf_addr + ELFHDR.e_phoff;
        let phs: &[Proghdr] = unsafe { from_raw_parts(ph_addr as * const usize as * const Proghdr, ELFHDR.e_phnum as usize) };
		for ph in phs {
			readseg(ph.p_va & 0xFFFFFF, ph.p_memsz, ph.p_offset);
		}
		let kern_addr = (ELFHDR.e_entry & 0xFFFFFF) as usize;
		jump_to(kern_addr);
	}
	else {
	    outw(0x8A00, 0x8A00);
	    outw(0x8A00, 0x8E00);

	    /* do nothing */
	    loop {}
	}
}
