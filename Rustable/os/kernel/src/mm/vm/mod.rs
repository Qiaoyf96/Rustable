mod address;
pub mod page_fault;
pub use self::address::{PhysicalAddr, VirtualAddr};

use allocator::page::{
    PGSIZE, PT0X, 
    PT1X, PT2X, PT3X, PTE_ADDR, PTE_V, AF, 
    UXN, ATTRIB_AP_RW_EL1, ATTRIB_SH_INNER_SHAREABLE, MAXPA,
    ATTRINDX_DEVICE, ATTRINDX_NORMAL
};
use alloc::heap::{AllocErr, Layout};
use allocator::util::align_up;
use std;
use std::ptr;
use console::kprintln;
use ALLOCATOR;
use alloc::allocator::Alloc;
use allocator::alloc_page;

use pi::timer::{spin_sleep_ms};

extern "C" {
    static _end: u8;
}

pub static mut FREEMEM: usize = 0;

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn print_sth() {
    spin_sleep_ms(1000);
    kprintln!("finished vm_init");
}

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn vm_init() {
    spin_sleep_ms(1000);

    kprintln!("vm init");
    
    let mut binary_end = unsafe { (&_end as *const u8) as u32 };
    
    unsafe { FREEMEM = align_up(binary_end as usize, PGSIZE); }
    kprintln!("freemem: {:x}", unsafe{FREEMEM});

    // /* Step 1: Allocate a page for page directory(first level page table). */
    let mut pgdir = boot_alloc(PGSIZE, true).expect("out of memory");

    kprintln!("boot alloced a pgdir");

    /* Step 2: Allocate proper size of physical memory for global array `pages`,
     * for physical memory management. Then, map virtual address `UPAGES` to
     * physical address `pages` allocated before. For consideration of alignment,
     * you should round up the memory size before map. */
    //pages = (struct Page *)boot_alloc(npage * sizeof(struct Page), BY2PG, 1);

    //envs = (struct Env *)boot_alloc(NENV * sizeof(struct Env), BY2PG, 1);

    let n = align_up(MAXPA, PGSIZE);
    // let n = 4096 * 1000;
    kprintln!("n: {:x}", n);
    boot_map_segment(pgdir, 0, n, 0, ATTRINDX_NORMAL);
    kprintln!("boot map segment finished 1 ");
    boot_map_segment(pgdir, n, n, n, ATTRINDX_DEVICE);
    kprintln!("boot map segment finished 2 ");
}

fn boot_alloc(n: usize, clear: bool) -> Result<*mut usize, AllocErr> {
        
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
    if alloced_mem >= MAXPA {
        return Err( AllocErr::Unsupported { details: "no memory" } );
    }

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

pub fn get_pte(pgdir_addr: *const usize, va: usize, create: bool) -> Result<*mut usize, AllocErr> {
    // kprintln!("================= GET PTE =================");
    let pgdir = unsafe { std::slice::from_raw_parts_mut(pgdir_addr as *mut usize, 512) };
    // kprintln!("virtual address: {:x}", va);
    // kprintln!("FREEMEM: {:x}", unsafe { FREEMEM });
    
    let mut pgtable0_entry_ptr = &mut pgdir[PT0X(va)];
    // kprintln!("pgtable0: {:x} {:x}", PT0X(va), unsafe{ *pgtable0_entry_ptr });
    let mut pgtable1 = PTE_ADDR(*pgtable0_entry_ptr) + PT1X(va) * 8;
    if (*pgtable0_entry_ptr & PTE_V) == 0 && create == true {
        pgtable1 = alloc_page().expect("cannot alloc page") as usize;
        *pgtable0_entry_ptr = pgtable1 | PTE_V;
        pgtable1 += PT1X(va) * 8;
    }
    // kprintln!("pgtable1: {:x} {:x}", PT1X(va), pgtable1);
    let mut pgtable1_entry_ptr = pgtable1 as *mut usize;
    let mut pgtable2 = PTE_ADDR(unsafe{ *pgtable1_entry_ptr }) + PT2X(va) * 8;
    if (unsafe{ *pgtable1_entry_ptr & PTE_V }) == 0 && create == true {
        pgtable2 = alloc_page().expect("cannot alloc page") as usize;
        unsafe{ *pgtable1_entry_ptr = pgtable2 | PTE_V; }
        pgtable2 += PT2X(va) * 8;
    }
    // kprintln!("pgtable2: {:x} {:x}", PT2X(va), pgtable2);
    let mut pgtable2_entry_ptr = pgtable2 as *mut usize;
    let mut pgtable3 = PTE_ADDR(unsafe{ *pgtable2_entry_ptr }) + PT3X(va) * 8;
    if (unsafe{ *pgtable2_entry_ptr & PTE_V }) == 0 && create == true {
        pgtable3 = alloc_page().expect("cannot alloc page") as usize;
        unsafe{ *pgtable2_entry_ptr = pgtable3 | PTE_V; }
        pgtable3 += PT3X(va) * 8;
    } else if (unsafe{ *pgtable2_entry_ptr & PTE_V }) == 0 {
        return Err( AllocErr::Unsupported { details: "get pte failed" } );
    }
    // kprintln!("pgtable3: {:x} {:x}", PT3X(va), pgtable3);
    /* Step 3: Get the page table entry for `va`, and return it. */
    // kprintln!("==========================================");
    Ok(pgtable3 as *mut usize)
}
