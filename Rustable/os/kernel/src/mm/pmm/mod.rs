use ALLOCATOR;
use std;
use std::mem;
use allocator::util::{align_down, align_up};
use alloc::heap::{AllocErr, Layout};
// use pi::atags;
use pi::atags::Atags;

mod page_table;

use self::page_table::boot_alloc_page;

use allocator::page::{PGSIZE, Page, PPN, MAXPA, VPN};


use console::kprintln;

pub struct Pmm;



impl Pmm {
    pub fn init(&self) {
        // to alloc/dealloc physical memory
        // detect physical memory space, reservery already used memory,
        // create free page list
        ALLOCATOR.initialize();

        kprintln!("Allocator initialized!");
        
        page_init();

        // // use create boot_pgdir, an initial page directory 
        // let page_table_ptr = boot_alloc_page().expect("No page allocated");
        // // memset boot_pgdir 0
        // let page_table = &mut *page_table_ptr;

        // page_table.clear();
        // // boot_cr3 = PADDR(boot_pgdir);
        // page_table.kva = page_table as *mut usize as usize;   
        // kprintln!("Page Table kva: {:x}", page_table.kva);

        // // fill in the page table
        // page_table[PDX(VPT)] = page_table_kva;

        // n = align_up(MAXPA, PGSIZE)
        // page_table.boot_map_segment(page_table_kva, 0, n, 0, ATTRINDX_NORMAL);
        // page_table.boot_map_segment(page_table_kva, n, n, n, ATTRINDX_NORMAL);

        // enable paing 
    }
}

extern "C" {
    static _end: u8;
}

fn VADDR(kaddr: usize) -> usize {
    (kaddr + 0xffffff0000000000) as usize
}

fn PADDR(vaddr: usize) -> usize {
    (vaddr - 0xffffff0000000000) as usize
}

fn page_init() {
    let binary_end = unsafe { (&_end as *const u8) as usize };
    let binary_end_val = unsafe { *(&_end as *const u8 as *const usize) };
    kprintln!("Binary_end: {:x} {:x}", binary_end, binary_end_val);
    let mut maxpa = 0 as usize;
    let PMEMSIZE = (512 * 1024 * 1024) as usize;
    for atag in Atags::get() {
        match atag.mem() {
            Some(mem) => {
                let begin = mem.start as usize;
                let end = mem.size as usize;
                kprintln!("mem: {:x} {:x}", begin, end);
                if maxpa < end && begin < PMEMSIZE {
                    maxpa = end;
                }
            },
            None => {}
        }
    }
    if maxpa > PMEMSIZE {
        maxpa = PMEMSIZE;
    }
    let npage = maxpa / PGSIZE;
    kprintln!("number of pages: {}", npage);
    let pages = align_up(binary_end as usize, PGSIZE) as *mut Page;

    ALLOCATOR.init_page_list(pages as *mut usize as usize, npage);

    // set page reserved
    let page = unsafe { std::slice::from_raw_parts_mut(pages, npage) };
    for i in 0..npage {
        page[i].SetPageReserved();
    }

    let FREEMEM = (pages as usize) + mem::size_of::<Page>() * npage;
    kprintln!("FREEMEM: {:x}", FREEMEM);
    kprintln!("PMEMSIZE: {:x}", PMEMSIZE);
    for atag in Atags::get() {
        match atag.mem() {
            Some(mem) => {
                let mut begin = VADDR(mem.start as usize);
                let mut end = VADDR(mem.size as usize);
                kprintln!("mem2: {:x} {:x}", begin, end);
                if begin < FREEMEM {
                    begin = FREEMEM;
                }
                // if end > PMEMSIZE {
                //     end = PMEMSIZE;
                // }
                kprintln!("mem3: {:x} {:x}", begin, end);
                if begin < end {
                    begin = align_up(begin, PGSIZE);
                    end = align_down(end, PGSIZE);
                    kprintln!("mem4: {:x} {:x}", begin, end);
                    let page_addr = &page[VPN(begin)] as *const Page as *mut usize as usize;
                    kprintln!("page addr {:x}", page_addr);
                    if begin < end {
                        ALLOCATOR.init_memmap(page_addr, (end - begin) / PGSIZE, begin);
                        // init_memmap(struct Page *base, size_t n) {
                        //     pmm_manager->init_memmap(base, n);
                        // }
                    }
                }
            }

            None => {}
        }
    }

}
