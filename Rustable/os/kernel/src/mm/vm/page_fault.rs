use traps::syndrome::Fault;
use console::kprintln;
use aarch64::{get_far, get_ttbr0};
use mm::pmm::{page_insert};
use mm::vm::{get_pte};
use allocator::page::{ATTRIB_AP_RW_ALL, ATTRIB_AP_RO_ALL, pa2page};

use allocator::alloc_page;

pub fn do_pgfault(kind: Fault, level: u8) {
    let va = unsafe { get_far() };
    kprintln!("pg_fault! {:?} {} {:x}", kind, level, va);

    let ttbr0 = unsafe { get_ttbr0() as *const usize };
    
    // pgdir_walk(curenv->env_ttbr0, va, 0, &pte);
    match get_pte(ttbr0, va, true) {
        Ok(pte) => {
            kprintln!("*pte: {:x}", unsafe {*pte});
            if unsafe{*pte & ATTRIB_AP_RO_ALL != 0 } {
                kprintln!("Succeed in get_pte, but it is not a copy-on-write page at va: {:x}\n", va);
                return;
            }
            kprintln!("~~~");
            let paddr = alloc_page().expect("cannot alloc page");
            page_insert( ttbr0 , pa2page(paddr as usize), va, ATTRIB_AP_RW_ALL);
            
        },
        Err(_) => {
            kprintln!("It is not a copy-on-write page at va: {:x}\n", va);
        }
    }
}