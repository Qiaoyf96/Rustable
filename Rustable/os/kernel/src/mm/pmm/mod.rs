use ALLOCATOR;

use allocator::page::Page;

pub struct Pmm;

const MAXPA: u32 = (512 * 1024 * 1024)

impl Pmm {
    pub const fn init() {
        // to alloc/dealloc physical memory
        // detect physical memory space, reservery already used memory,
        // create free page list
        ALLOCATOR.initialize();
        
        page_init();

        // // use create boot_pgdir, an initial page directory 
        // let page_table = boot_alloc_page()?;
        // // memset boot_pgdir 0
        // page_table.clear();
        // // boot_cr3 = PADDR(boot_pgdir);
        // page_table.kva = page_table as *usize as usize;   

        // // fill in the page table
        // page_table[PDX(VPT)] = page_table_kva;

        // n = align_up(MAXPA, PGSIZE)
        // page.table.boot_map_segment(page_table_kva, 0, n, 0, ATTRINDX_NORMAL);
        // page.table.boot_map_segment(page_table_kva, n, n, n, ATTRINDX_NORMAL);

        // enable paing 
    }
}

fn page_init() {
    let binary_end = unsafe { (&_end as *const u8) as u32 };
    let mut maxpa = 0 as usize;
    let PMEMSIZE = (512 * 1024 * 1024) as usize;
    for atag in Atags::get() {
        match atag.mem() {
            Some(mem) => {
                let begin = mem.start as usize;
                let end = mem.size as usize;
                if maxpa < end and begin < PMEMSIZE {
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
    pages = align_up(end, PGSIZE) as *mut Page;

    // set page reserved
    page = unsafe { std::slice::from_raw_parts_mut(pages, npage) };
    for i in 0..npage {
        page[i].SetPageReserved();
    }

    let FREEMEM = (pages as usize) + mem::size_of::<Page>() * npage;
    
    for atag in Atags::get() {
        match atag.mem() {
            Some(mem) => {
                let begin = mem.start as usize;
                let end = mem.size as usize;
                if begin < FREEMEM {
                    begin = FREEMEM;
                }
                if end > PMEMSIZE {
                    end = PMEMSIZE;
                }
                if begin < end {
                    begin = align_up(begin, PGSIZE);
                    end = align_down(begin, PGSIZE);
                    let page_addr = &page[PPN(begin)] as *mut usize;
                    if begin < end {
                        ALLOCATOR.init_memmap(page_addr, (end - begin) / PGSIZE);
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
