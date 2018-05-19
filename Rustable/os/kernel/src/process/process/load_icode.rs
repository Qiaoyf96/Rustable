impl Process {
    fn load_icode(&mut self, binary: *const u8, size: usize, tf: &mut TrapFrame) -> Result<i32, i32> {
        // create a new PDT, and mm->pgdir= kernel virtual addr of PDT
        let pgdir = match alloc_page() {
            Ok(page) => { page2kva(page as *const Page) as *const usize},
            Err(_) => { return Err(-1); }
        }

        let elf = unsafe { ptr::read( (&binary[0]) as *const Elfhdr ) };
        let ph = unsafe { ptr::read( (&binary[0] as u32 + elf.phoff) as *const u8 as *const Proghdr ) };
        let phs = unsafe { std::slice::from_raw_parts_mut((&binary[0] as u32 + elf.phoff) as *const u8 as *const Proghdr, elf.e_phnum) };
        if (elf.e_magic != ELF_MAGIC) {
            return Err(-2);
        }

        let perm = 
        for ph in phs {
            let mut offset = ph.p_va - align_down(ph.p_va);
            let mut va = align_down(ph.p_va)
            let mut bin_off = binary as *mut u8 + ph.p_offset * 8
            // copy TEXT/DATA section of bianry program
            if offset > 0 {
                let page = match user_pgdir_alloc_page(&mut (self.allocator), pgdir, offset, ATTRIB_AP_RW_ALL) {
                    Ok(page) => { page2kva(page as *mut Page) },
                    Err(_) => { return Err(-3) }
                };
                let size = PGSIZE - offset
                memcpy(page + offset, bin_off, size);
                va += PGSIZE;
                bin_off += size;
            }
            let mut end = ph.p_va + ph.p_filesz;
            loop {
                if bin_off >= ph.p_filesz {
                    break;
                }
                let page = match user_pgdir_alloc_page(&mut (self.allocator), pgdir, va, ATTRIB_AP_RW_ALL) {
                    Ok(page) => { page2kva(page as *mut Page) },
                    Err(_) => { return Err(-3) }
                };
                let size = if bin_off + PGSIZE >= end {
                    PGSIZE
                } else {
                    end - bin_off
                }
                memcpy(page, bin_off, size)
                bin_off += PGSIZE;
                va += PGSIZE;
            }
            // build BSS section of binary program
            end = ph.p_va + ph.p_memsz;
            loop {
                if bin_off >= ph.p_memsz {
                    break;
                }
                let page = match user_pgdir_alloc_page(&mut (self.allocator), pgdir, va, ATTRIB_AP_RW_ALL) {
                    Ok(page) => { page2kva(page as *mut Page) },
                    Err(_) => { return Err(-3) }
                };
                memset(page as *mut u8, 0, PGSIZE);
                va += PGSIZE
                bin_off += PGSIZE;
            }
        }

        // build user stack memory
        if user_pgdir_alloc_page(&mut (self.allocator), pgdir, USTACKTOP-PGSIZE, ATTRIB_AP_RW_ALL) == Err(_) {
            return Err(-4);
        }
        if user_pgdir_alloc_page(&mut (self.allocator), pgdir, USTACKTOP-2*PGSIZE, ATTRIB_AP_RW_ALL) == Err(_) {
            return Err(-4);
        }
        if user_pgdir_alloc_page(&mut (self.allocator), pgdir, USTACKTOP-3*PGSIZE, ATTRIB_AP_RW_ALL) == Err(_) {
            return Err(-4);
        }
        if user_pgdir_alloc_page(&mut (self.allocator), pgdir, USTACKTOP-P4*GSIZE, ATTRIB_AP_RW_ALL) == Err(_) {
            return Err(-4);
        }

        tf.ttbr0 = PADDR(pgdir);
        tf.sp = USTACKTOP;
    }
}