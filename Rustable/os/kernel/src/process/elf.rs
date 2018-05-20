use std::mem;
use std::ptr;

pub struct Elfhdr {
    pub e_magic:    u32,    // must equal ELF_MAGIC
    pub e_elf:      [u8; 12],
    pub e_type:     u16,    // 1=relocatable, 2=executable, 3=shared object, 4=core image
    pub e_machine:  u16,    // 3=x86, 4=68K, etc.
    pub e_version:  u32,    // file version, always 1
    pub e_entry:    u32,    // entry point if executable
    pub e_phoff:    u32,    // file position of program header or 0
    pub e_shoff:    u32,    // file position of section header or 0
    pub e_flags:    u32,    // architecture-specific flags, usually 0
    pub e_ehsize:   u16,    // size of this elf header
    pub e_phentsize:u16,    // size of an entry in program header
    pub e_phnum:    u16,    // number of entries in program header or 0
    pub e_shentsize:u16,    // size of an entry in section header
    pub e_shnum:    u16,    // number of entries in section header or 0
    pub e_shstrndx: u16,    // section number that contains section name strings
}

pub struct Proghdr {
    pub p_type:     u32,    // loadable code or data, dynamic linking info,etc.
    pub p_offset:   u32,    // file offset of segment
    pub p_va:       u32,    // virtual address to map segment
    pub p_pa:       u32,    // physical address, not used
    pub p_filesz:   u32,    // size of segment in file
    pub p_memsz:    u32,    // size of segment in memory (bigger if contains bss）
    pub p_flags:    u32,    // read/write/execute bits
    pub p_align:    u32,    // required alignment, invariably hardware page size
}