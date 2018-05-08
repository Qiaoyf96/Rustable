// ARM definitions.
const PGSIZE: u32 = 4096;

// index of page table entry
fn PT0X(va: usize) -> usize { (va >> 39) & 0x01 }
fn PT1X(va: usize) -> usize { (va >> 30) & 0x1FF }
fn PT2X(va: usize) -> usize { (va >> 21) & 0x1FF }
fn PT3X(va: usize) -> usize { (va >> 12) & 0x1FF }

// gets addr of pte from pte with modifier
fn PTE_ADDR(pte: usize) -> usize { pte & 0xFFFFFFF000 }

// page number field of address
fn PPN(va: usize) -> usize { va >> 12 }
fn VPN(va: usize) -> usize { (va & 0xFFFFFFFFFF) >> 12 }
fn PPN(va: usize) -> usize { va >> 12 }
fn VPN(va: usize) -> usize { (va & 0xFFFFFFFFFF) >> 12 }
const PGSHIFT: usize = 12
fn KADDR(pa) -> usize { pa | 0xFFFFFF0000000000 }
fn VA2PFN(va)-> usize { va & 0xFFFFFFFFF000 } // va 2 PFN for EntryLo0/1
const PTE2PT: usize = 512

// Page Table/Directory Entry flags
// these are defined by the hardware
const PTE_V: usize = 0x3 << 0    // Table Entry Valid bit
const PBE_V: usize = 0x1 << 0    // Block Entry Valid bit
const ATTRIB_AP_RW_EL1: usize = 0x0 << 6
const ATTRIB_AP_RW_ALL: usize = 0x1 << 6
const ATTRIB_AP_RO_EL1: usize = 0x2 << 6
const ATTRIB_AP_RO_ALL: usize = 0x3 << 6
const ATTRIB_SH_NON_SHAREABLE: usize = 0x0 << 8
const ATTRIB_SH_OUTER_SHAREABLE: usize = 0x2 << 8
const ATTRIB_SH_INNER_SHAREABLE: usize = 0x3 << 8
const AF: usize = 0x1 << 10
const PXN: usize = 0x0 << 53
const UXN: usize = 0x1UL << 54

const ATTRINDX_NORMAL: usize = 0 << 2    // inner/outer write-back non-transient, non-allocating
const ATTRINDX_DEVICE: usize = 1 << 2    // Device-nGnRE
const ATTRINDX_COHERENT: usize = 2 << 2    // Device-nGnRnE




struct Page {
    list_entry: *mut usize,    // used for linked list
    reference: u32,           // page frame's reference counter
    flags: u32,         // array of flags that describe the status of the page frame
    property: u32,   // the num of free block
}

impl Page {
    fn SetPageReserved(&mut self) {
        self.flags = self.flags | 1;
    }

    fn SetPageProperty(&mut self) {
        self.flags = self.flags | 0b10;
    }

    fn ClearPageProperty(&mut self) {
        self.flags = self.flags & 0xfffffffd;
    }

    fn ClearPageReserved(&mut self) {
        self.flags = self.flags & 0xfffffffe;
    }

    fn set_page_ref(&mut self, val: u32) {
        self.reference = val;
    }
}
