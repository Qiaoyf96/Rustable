use mem;
use ptr;

struct Elfhdr {
    e_magic:    u32,    // must equal ELF_MAGIC
    e_elf:      [u8; 12],
    e_type:     u16,    // 1=relocatable, 2=executable, 3=shared object, 4=core image
    e_machine:  u16,    // 3=x86, 4=68K, etc.
    e_version:  u32,    // file version, always 1
    e_entry:    u32,    // entry point if executable
    e_phoff:    u32,    // file position of program header or 0
    e_shoff:    u32,    // file position of section header or 0
    e_flags:    u32,    // architecture-specific flags, usually 0
    e_ehsize:   u16,    // size of this elf header
    e_phentsize:u16,    // size of an entry in program header
    e_phnum:    u16,    // number of entries in program header or 0
    e_shentsize:u16,    // size of an entry in section header
    e_shnum:    u16,    // number of entries in section header or 0
    e_shstrndx: u16,    // section number that contains section name strings
}

struct Proghdr {
    p_type:     u32,    // loadable code or data, dynamic linking info,etc.
    p_offset:   u32,    // file offset of segment
    p_va:       u32,    // virtual address to map segment
    p_pa:       u32,    // physical address, not used
    p_filesz:   u32,    // size of segment in file
    p_memsz:    u32,    // size of segment in memory (bigger if contains bss）
    p_flags:    u32,    // read/write/execute bits
    p_align:    u32,    // required alignment, invariably hardware page size
}