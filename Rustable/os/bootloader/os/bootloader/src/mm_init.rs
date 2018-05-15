use std;
use std::ptr;

use pi::timer::{spin_sleep_ms};

// ARM definitions.
pub const PGSIZE: usize = 4096;
pub const MAXPA: usize = (512 * 1024 * 1024);
// index of page table entry
pub fn PT0X(va: usize) -> usize { (va >> 39) & 0x01 }
pub fn PT1X(va: usize) -> usize { (va >> 30) & 0x1FF }
pub fn PT2X(va: usize) -> usize { (va >> 21) & 0x1FF }
pub fn PT3X(va: usize) -> usize { (va >> 12) & 0x1FF }
// gets addr of pte from pte with modifier
pub fn PTE_ADDR(pte: usize) -> usize { pte & 0xFFFFFFF000 }
// page number field of address
pub fn PPN(va: usize) -> usize { va >> 12 }
pub fn VPN(va: usize) -> usize { (va & 0xFFFFFFFFFF) >> 12 }
pub const PGSHIFT: usize = 12;
pub fn KADDR(pa: usize) -> usize { pa | 0xFFFFFF0000000000 }
pub fn VA2PFN(va: usize)-> usize { va & 0xFFFFFFFFF000 } // va 2 PFN for EntryLo0/1
pub const PTE2PT: usize = 512;
// Page Table/Directory Entry flags
pub const PTE_V: usize = 0x3 << 0;    // Table Entry Valid bit
pub const PBE_V: usize = 0x1 << 0;    // Block Entry Valid bit
pub const ATTRIB_AP_RW_EL1: usize = 0x0 << 6;
pub const ATTRIB_AP_RW_ALL: usize = 0x1 << 6;
pub const ATTRIB_AP_RO_EL1: usize = 0x2 << 6;
pub const ATTRIB_AP_RO_ALL: usize = 0x3 << 6;
pub const ATTRIB_SH_NON_SHAREABLE: usize = 0x0 << 8;
pub const ATTRIB_SH_OUTER_SHAREABLE: usize = 0x2 << 8;
pub const ATTRIB_SH_INNER_SHAREABLE: usize = 0x3 << 8;
pub const AF: usize = 0x1 << 10;
pub const PXN: usize = 0x0 << 53;
pub const UXN: usize = 0x1 << 54;
pub const ATTRINDX_NORMAL: usize = 0 << 2;    // inner/outer write-back non-transient, non-allocating
pub const ATTRINDX_DEVICE: usize = 1 << 2;    // Device-nGnRE
pub const ATTRINDX_COHERENT: usize = 2 << 2;    // Device-nGnRnE

// pub struct Page {
//     pub list_entry: LinkedList,    // used for linked list
//     pub reference: u32,           // page frame's reference counter
//     pub flags: u32,         // array of flags that describe the status of the page frame
//     pub property: u32,   // the num of free block
// }

pub fn align_down(addr: usize, align: usize) -> usize {
    if align == 0 || align & (align - 1) > 0 { panic!("ERROR: Align is not power of 2"); }
    addr / align * align
}

pub fn align_up(addr: usize, align: usize) -> usize {
    if align == 0 || align & (align - 1) > 0 { panic!("ERROR: Align is not power of 2"); }
    (addr + align - 1) / align * align
}

extern "C" {
    static _end: u8;
}

pub static mut FREEMEM: usize = 0;

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn vm_init() {
    // spin_sleep_ms(1000);

    // kprintln!("vm init");
    
    let mut binary_end = 0x1000000;
    
    unsafe { FREEMEM = align_up(binary_end as usize, PGSIZE); }
    // kprintln!("freemem: {:x}", unsafe{FREEMEM});

    // /* Step 1: Allocate a page for page directory(first level page table). */
    let mut pgdir = boot_alloc(PGSIZE, true).expect("out of memory");

    // kprintln!("boot alloced a pgdir");

    /* Step 2: Allocate proper size of physical memory for global array `pages`,
     * for physical memory management. Then, map virtual address `UPAGES` to
     * physical address `pages` allocated before. For consideration of alignment,
     * you should round up the memory size before map. */
    //pages = (struct Page *)boot_alloc(npage * sizeof(struct Page), BY2PG, 1);

    //envs = (struct Env *)boot_alloc(NENV * sizeof(struct Env), BY2PG, 1);

    let n = align_up(MAXPA, PGSIZE);
    // let n = 4096 * 1000;
    // kprintln!("n: {:x}", n);
    boot_map_segment(pgdir, 0, n, 0, ATTRINDX_NORMAL);
    // kprintln!("boot map segment finished 1 ");
    boot_map_segment(pgdir, n, n, n, ATTRINDX_DEVICE);
    // kprintln!("boot map segment finished 2 ");
}

fn boot_alloc(n: usize, clear: bool) -> Result<*mut usize, i32> {
        
    let alloced_mem = unsafe { FREEMEM };
    unsafe { FREEMEM += n; }
    // kprintln!("freemem: {:x}", unsafe{FREEMEM});
    /* Step 4: Clear allocated chunk if parameter `clear` is set. */
    if clear {
        let bytes = unsafe { std::slice::from_raw_parts_mut(alloced_mem as *mut u8, n) };
        for i in 0..n {
            bytes[i] = 0;
        }
    }
    // We're out of memory, PANIC !!
    // if (alloced_mem >= MAXPA) {
    //     return Err(-1);
    // }

    /* Step 5: return allocated chunk. */
    Ok(alloced_mem as *mut usize)
}

fn boot_map_segment(pgdir: *mut usize, _va: usize, _size: usize, _pa: usize, perm: usize) {
    let mut va = align_up(_va, PGSIZE);
    let mut pa = align_up(_pa, PGSIZE);
    let mut size = _size;
    loop {
        // kprintln!("boot map: {:x} {:x} {:x}", va, pa, size);
        let ptep = match get_pte(pgdir, va, true) { // return *pte = *mut usize
            Ok(p) => p,
            Err(_) => ptr::null_mut(),
        };
        // kprintln!("get pte finished {:x}.", ptep as usize);
        // kprintln!("get pte: {:x} content: {:x}", ptep as usize, unsafe{ *ptep });
        unsafe{ *ptep = PTE_ADDR(pa) | perm | PTE_V | ATTRIB_AP_RW_EL1 | ATTRIB_SH_INNER_SHAREABLE | AF | UXN; }
        va += PGSIZE;
        pa += PGSIZE;
        size -= PGSIZE;
        if size <= 0 {
            // kprintln!("break size: {:x}", size);
            break;
        }
    }
}

fn get_pte(pgdir_addr: *mut usize, va: usize, create: bool) -> Result<*mut usize, i32> {
    let pgdir = unsafe { std::slice::from_raw_parts_mut(pgdir_addr as *mut usize, 512) };
    // kprintln!("virtual address: {:x}", va);
    // kprintln!("FREEMEM: {:x}", unsafe { FREEMEM });
    let mut pgtable0_entry_ptr = &mut pgdir[PT0X(va)];
    // kprintln!("pgtable0: {:x} {:x}", PT0X(va), unsafe{ *pgtable0_entry_ptr });
    let mut pgtable1 = PTE_ADDR(unsafe{ *pgtable0_entry_ptr }) + PT1X(va) * 8;
    if (unsafe{ *pgtable0_entry_ptr & PTE_V }) == 0 && create == true {
        pgtable1 = boot_alloc(PGSIZE, true).expect("boot alloc falied") as usize;
        unsafe{ *pgtable0_entry_ptr = pgtable1 | PTE_V; }
        pgtable1 += PT1X(va) * 8;
    }
    // kprintln!("pgtable1: {:x} {:x}", PT1X(va), pgtable1);
    let mut pgtable1_entry_ptr = pgtable1 as *mut usize;
    let mut pgtable2 = PTE_ADDR(unsafe{ *pgtable1_entry_ptr }) + PT2X(va) * 8;
    if (unsafe{ *pgtable1_entry_ptr & PTE_V }) == 0 && create == true {
        pgtable2 = boot_alloc(PGSIZE, true).expect("boot alloc falied") as usize;
        unsafe{ *pgtable1_entry_ptr = pgtable2 | PTE_V; }
        pgtable2 += PT2X(va) * 8;
    }
    // kprintln!("pgtable2: {:x} {:x}", PT2X(va), pgtable2);
    let mut pgtable2_entry_ptr = pgtable2 as *mut usize;
    let mut pgtable3 = PTE_ADDR(unsafe{ *pgtable2_entry_ptr }) + PT3X(va) * 8;
    if (unsafe{ *pgtable2_entry_ptr & PTE_V }) == 0 && create == true {
        pgtable3 = boot_alloc(PGSIZE, true).expect("boot alloc falied") as usize;
        unsafe{ *pgtable2_entry_ptr = pgtable3 | PTE_V; }
        pgtable3 += PT3X(va) * 8;
    } else if (unsafe{ *pgtable2_entry_ptr & PTE_V }) == 0 {
        return Err(-1);
    }
    // kprintln!("pgtable3: {:x} {:x}", PT3X(va), pgtable3);
    /* Step 3: Get the page table entry for `va`, and return it. */
    Ok(pgtable3 as *mut usize)
}
