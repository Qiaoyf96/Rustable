use allocator::linked_list::LinkedList;
use std;
use std::mem;

// ARM definitions.
pub const PGSIZE: usize = 4096;
pub const MAXPA: usize = (512 * 1024 * 1024);
pub const NPAGE: usize = MAXPA / PGSIZE;
pub const KERNEL_PAGES: usize = 0xFFFFFF0000000000 + 0x01400000;

pub const USER_PAGES: usize = 0x1fd00000;
pub const USER_NPAGE: usize = 768;

pub const USTACKTOP: usize = 0x1f000000;

// index of page table entry
pub fn PT0X(va: usize) -> usize { (va >> 39) & 0x01 }
pub fn PT1X(va: usize) -> usize { (va >> 30) & 0x1FF }
pub fn PT2X(va: usize) -> usize { (va >> 21) & 0x1FF }
pub fn PT3X(va: usize) -> usize { (va >> 12) & 0x1FF }

// gets addr of pte from pte with modifier
pub fn PTE_ADDR(pte: usize) -> usize { pte & 0xFFFFFFF000 }
pub fn OFFSET(addr: usize) -> usize { addr & 0xFFF }

// page number field of address
pub fn PPN(va: usize) -> usize { va >> 12 }
pub fn VPN(va: usize) -> usize { (va & 0xFFFFFFFFFF) >> 12 }
pub const PGSHIFT: usize = 12;
pub fn KADDR(pa: usize) -> usize { pa | 0xFFFFFF0000000000 }
pub fn PADDR(va: usize) -> usize { va & 0x000000FFFFFFFFFF }
pub fn VA2PFN(va: usize)-> usize { va & 0xFFFFFFFFF000 } // va 2 PFN for EntryLo0/1
pub const PTE2PT: usize = 512;

// Page Table/Directory Entry flags
// these are defined by the hardware
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

pub fn page2ppn(page: *const Page) -> usize {
    (page as *const usize as usize - KERNEL_PAGES) / mem::size_of::<Page>()
}

pub fn page2pa(page: *mut Page) -> usize {
    page2ppn(page) << PGSHIFT
}

pub fn page2kva(page: *const Page) -> usize {
    KADDR(page2ppn(page) << PGSHIFT)
}

pub fn pa2page(pa: usize) -> *mut Page {
    if PPN(pa) >= NPAGE {
        panic!("pa2page called with invalid pa: {:x}", pa);
    }
    let pages = unsafe { std::slice::from_raw_parts_mut(KERNEL_PAGES as *mut usize as *mut Page, NPAGE) };
    &mut pages[PPN(pa)] as *mut Page
}

pub fn page2va(page: *const Page) -> usize {
    ((page as *const usize as usize - USER_PAGES) / mem::size_of::<Page>()) << PGSHIFT
}

pub fn user_pa2page(pa: usize) -> *mut Page {
    if PPN(pa) >= USER_NPAGE {
        panic!("pa2page called with invalid pa: {:x}", pa);
    }
    let pages = unsafe { std::slice::from_raw_parts_mut(USER_PAGES as *mut usize as *mut Page, USER_NPAGE) };
    &mut pages[PPN(pa)] as *mut Page
}

#[repr(C)]
pub struct Page {
    pub list_entry: LinkedList,    // used for linked list
    pub reference: i32,           // page frame's reference counter
    pub flags: u32,         // array of flags that describe the status of the page frame
    pub property: u32,   // the num of free block
}

impl Page {

    pub fn SetPageReserved(&mut self) {
        self.flags = self.flags | 1;
    }

    pub fn SetPageProperty(&mut self) {
        self.flags = self.flags | 0b10;
    }

    pub fn SetPageUsed(&mut self) {
        self.flags = self.flags | 0b100;
    }
    
    pub fn isUsed(&mut self) -> bool {
        (self.flags >> 2) & 0x1 == 1
    }

    pub fn ClearPageProperty(&mut self) {
        self.flags = self.flags & 0xfffffffd;
    }

    pub fn ClearPageReserved(&mut self) {
        self.flags = self.flags & 0xfffffffe;
    }

    pub fn ClearPageUsed(&mut self) {
        self.flags = self.flags & 0xfffffffb;
    }

    pub fn set_page_ref(&mut self, val: i32) {
        self.reference = val;
    }

    pub fn page_ref_dec(&mut self) -> i32 {
        self.reference -= 1;
        self.reference
    }

    pub fn page_ref_inc(&mut self) -> i32 {
        self.reference += 1;
        self.reference
    }
}
