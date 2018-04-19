
#[derive(Clone)]
pub struct Elfhdr {
	pub e_magic: u32,
	pub e_elf: [u8; 12],
	pub e_type: u16,
	pub e_machine: u16,
	pub e_version: u32,
	pub e_entry: usize,
	pub e_phoff: usize,
	pub e_shoff: usize,
	pub e_flags: u32,
	pub e_ehsize: u16,
	pub e_phentsize: u16,
	pub e_phnum: u16,
	pub e_shentsize: u16,
	pub e_shnum: u16,
	pub e_shstrndx: u16
}

pub struct Proghdr {
	pub p_type: u32,
	pub p_offset: u32,
	pub p_va: u32,
	pub p_pa: u32,
	pub p_filesz: u32,
	pub p_memsz: u32,
	pub p_flags: u32,
	pub p_align: u32
}