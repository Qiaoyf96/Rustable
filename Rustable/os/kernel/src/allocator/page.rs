use allocator::linked_list::LinkedList;

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




pub struct Page {
    pub list_entry: LinkedList,    // used for linked list
    pub reference: u32,           // page frame's reference counter
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

    pub fn ClearPageProperty(&mut self) {
        self.flags = self.flags & 0xfffffffd;
    }

    pub fn ClearPageReserved(&mut self) {
        self.flags = self.flags & 0xfffffffe;
    }

    pub fn set_page_ref(&mut self, val: u32) {
        self.reference = val;
    }
}
