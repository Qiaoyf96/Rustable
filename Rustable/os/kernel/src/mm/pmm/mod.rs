use ALLOCATOR;
use std;
use std::mem;
use mutex::Mutex;
use allocator::util::{align_down, align_up};
use alloc::heap::{AllocErr, Layout};
use alloc::allocator::Alloc;
use allocator::page::{pa2page, page2pa, PADDR, PTE_ADDR, PTE_V, AF, ATTRIB_SH_INNER_SHAREABLE, ATTRINDX_NORMAL, KERNEL_PAGES};
use mm::vm::get_pte;
use aarch64::tlb_invalidate;

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

fn page_init() {
    let binary_end = unsafe { &_end as *const u8 as u8 };
    // let binary_end_val = unsafe { *(&_end as *const u8 as *const usize) };
    // kprintln!("Binary_end: {:x} {:x}", binary_end, binary_end_val);
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
    let pages = align_up(KERNEL_PAGES, PGSIZE) as *mut Page;

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
                let mut begin = mem.start as usize;
                let mut end = mem.size as usize;
                kprintln!("mem2: {:x} {:x}", begin, end);
                if begin < PADDR(FREEMEM) {
                    begin = PADDR(FREEMEM);
                }
                // if begin < binary_end {
                //     begin = binary_end
                // }
                // if end > PMEMSIZE {
                //     end = PMEMSIZE;
                // }
                kprintln!("mem3: {:x} {:x}", begin, end);
                if begin < end {
                    begin = align_up(begin, PGSIZE);
                    end = align_down(end, PGSIZE);
                    kprintln!("mem4: {:x} {:x}", begin, end);
                    let page_addr = &page[PPN(begin)] as *const Page as *mut usize as usize;
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

pub fn page_insert(pgdir: *const usize, page: *mut Page, va: usize, perm: usize) -> Result<i32, i32>{
    let PERM = perm | PTE_V | ATTRINDX_NORMAL | ATTRIB_SH_INNER_SHAREABLE | AF;
    let mut pte: *mut usize;
    match get_pte(pgdir, va, false) {
        Ok(pte) => {
            (unsafe { &mut *page }).page_ref_inc();
            if unsafe{ *pte & PTE_V != 0} {
                if pa2page(PTE_ADDR(unsafe{*pte})) != page {
                    page_remove(pgdir, va, pte);
                } else {
                    (unsafe { &mut *page }).page_ref_dec();
                }
            }
            unsafe{ *pte = PTE_ADDR(page2pa(page)) | PERM };
            unsafe{ tlb_invalidate(va) };
            return Ok(0);
        },
        Err(_) => {
            return Err(-1);
        }
    }
    Err(-1)
}



pub fn page_remove(pgdir: *const usize, va: usize, pte: *mut usize) {
    let page = pa2page(pte as usize);
    if (unsafe { &mut *page }).page_ref_dec() <= 0 {
        // free_page(page);
        unsafe { (&ALLOCATOR).dealloc(pte as *mut u8, Layout::from_size_align_unchecked(PGSIZE, PGSIZE)); }
    }
    unsafe { *pte = 0; }
    unsafe{ tlb_invalidate(va) };
}
