use ALLOCATOR;
use alloc::heap::{AllocErr, Layout};
use allocator::page::PGSIZE;
use alloc::allocator::Alloc;

pub struct PageTable {
    // pte_list: [pte; 4096],
    pub kva: usize,
}

impl PageTable {
    fn clear() {
        // for i in 0..4096 {
        //     self.pte_list[i] = 0;
        // }
    }

    // /*Overview:
    //  *  Map [va, va+size) of virtual address space to physical [pa, pa+size) in the page
    //  *  table rooted at pgdir.
    //  *  Use permission bits `perm|PTE_V` for the entries.
    //  *  Use permission bits `perm` for the entries.
    fn boot_map_segment(pgdir: usize, size: usize, va: usize, pa: usize, perm: i32) {
        // let n = align_up(size + PGOFF(la), PGSIZE) / PGSIZE;
        // la = align_up(la, PGSIZE);
        // pa = align_up(pa, PGSIZE);
        // loop {
        //     let ptep = match get_pte(pgdir, la, 1) { // return *pte
        //         Ok(p) => p,
        //         Err(_) => 
        //     };
        //     unsafe{ &*(ptep) = PTE_ADDR(pa) | perm | PTE_V | ATTRIB_AP_RW_EL1 | ATTRIB_SH_INNER_SHAREABLE | AF | UXN };
        //     va += PGSIZE;
        //     pa += PGSIZE;
        //     size -= PGSIZE;
        //     if size == 0 {
        //         break;
        //     }
        // }
    }
}

pub fn boot_alloc_page() -> Result<*mut PageTable, AllocErr> {
    let layout = unsafe { Layout::from_size_align_unchecked(PGSIZE, PGSIZE)} ;
    match unsafe {(&ALLOCATOR).alloc(layout)} {
        Ok(p) => Ok( unsafe { p as *mut PageTable } ),
        Err(err) => Err(err),
    }
}