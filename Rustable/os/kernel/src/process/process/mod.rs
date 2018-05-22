pub mod utils;

use traps::TrapFrame;
use process::State;
use std::string::String;

use allocator::imp::Allocator;
use allocator::util::{align_down};
use allocator::{alloc_page};
use mm::pmm::user_pgdir_alloc_page;
use allocator::page::{ Page, PGSIZE, USTACKTOP, PADDR, page2kva, page2va, ATTRIB_AP_RW_ALL, KADDR };

pub const PXN: usize = 0x1 << 53;
pub const UXN: usize = 0x0 << 54;

use self::utils::{memset, memcpy};
use process::elf::{Elfhdr, Proghdr};
// use console;
use std::mem;
use std;
use std::ptr;

use mm::vm::get_pte;
use console::kprintln;

/// Type alias for the type of a process ID.
pub type Id = u64;


pub static mut pid: usize = 0;
pub const ELF_MAGIC: u32 = 0x464C457F;
fn get_unique_pid() -> usize {
    unsafe{ pid += 1 };
    unsafe{ pid - 1 }
}
/// A structure that represents the complete state of a process.
#[derive(Debug)]
pub struct Process {
    /// The saved trap frame of a process.
    pub trap_frame: Box<TrapFrame>,
    /// The memory allocation used for the process's stack.
    // pub stack: Stack,
    /// The scheduling state of the process.
    pub state: State,
    pub proc_name: String,
    pub allocator: Allocator,
    pub pid: usize,
    pub parent: &Process
}

impl Process {
    /// Creates a new process with a zeroed `TrapFrame` (the default), a zeroed
    /// stack of the default size, and a state of `Ready`.
    ///
    /// If enough memory could not be allocated to start the process, returns
    /// `None`. Otherwise returns `Some` of the new `Process`.
    pub fn new() -> Process {
        Self {
            trap_frame: Box::new(TrapFrame::default()),
            // stack,
            state: State::Ready,
            allocator: Allocator::new(),
            pid: 0,
            proc_name: String::from("idle"),
        }
    }

    pub fn proc_init(&mut self) {
        self.pid = get_unique_pid();
        // self.allocator.init_user();
    }

    pub fn set_proc_name(&mut self, s: &str) {
        self.proc_name = String::from(s);
    }

    pub fn get_id(&self) -> u64 {
        self.trap_frame.tpidr
    }

    /// Returns `true` if this process is ready to be scheduled.
    ///
    /// This functions returns `true` only if one of the following holds:
    ///
    ///   * The state is currently `Ready`.
    ///
    ///   * An event being waited for has arrived.
    ///
    ///     If the process is currently waiting, the corresponding event
    ///     function is polled to determine if the event being waiting for has
    ///     occured. If it has, the state is switched to `Ready` and this
    ///     function returns `true`.
    ///
    /// Returns `false` in all other cases.
    pub fn is_ready(&mut self) -> bool {
        if let State::Ready = self.state {
            true
        } else if let State::Running = self.state {
            false
        } else {
            let state = mem::replace(&mut self.state, State::Ready);
            if let State::Waiting(mut event_poll_fn) = state {
                if event_poll_fn(self) {
                    true
                } else {
                    self.state = State::Waiting(event_poll_fn);
                    false
                }
            } else {
                unreachable!();
            }
        }
    }
    
}

impl Process {
    pub fn load_icode(&mut self, binary: *mut u8, size: usize) -> Result<i32, i32> {
        kprintln!("================ LOAD_ICODE ================");
        // create a new PDT, and mm->pgdir= kernel virtual addr of PDT
        let pgdir = match alloc_page() {
            Ok(paddr) => { KADDR(paddr as usize) as *const usize},
            Err(_) => { return Err(-1); }
        };

        self.allocator.init_user(pgdir);

        kprintln!("finish init user page");
        
        kprintln!("content:");
        let bits = unsafe { std::slice::from_raw_parts_mut(binary, 1000) };
        kprintln!("{}", String::from_utf8_lossy(&bits));
        kprintln!("end");

        kprintln!("binary: {:x}", binary as usize);
        let elf = unsafe { ptr::read( binary as *const Elfhdr ) };

        kprintln!("e_magic:   {:x}", elf.e_magic);
        kprintln!("e_type:    {:x}", elf.e_type);
        kprintln!("e_machine: {:x}", elf.e_machine);
        kprintln!("e_version: {:x}", elf.e_version);
        kprintln!("elf phoff: {}, elf phnum: {}, elf entry: {}, shoff: {}, shnum: {} ", elf.e_phoff, elf.e_phnum, elf.e_entry, elf.e_shoff, elf.e_shnum);
        
        let phs = unsafe { std::slice::from_raw_parts_mut( binary.add(elf.e_phoff as usize) as *mut Proghdr, elf.e_phnum as usize) };
        if elf.e_magic != ELF_MAGIC {
            kprintln!("not elf");
            return Err(-2);
        }
        let perm = UXN | PXN | ATTRIB_AP_RW_ALL;
        let mut ph_idx = 0;
        for ph in phs {
            kprintln!("ph idx: {}", ph_idx);
            ph_idx += 1;

            let mut offset = ph.p_va as usize - align_down(ph.p_va as usize, PGSIZE);
            let mut va = align_down(ph.p_va as usize, PGSIZE) as usize;
            let mut bin_off = ph.p_offset as usize;
            kprintln!("va: {:x} p_offset: {} filesz: {} memsz: {} offset: {}", ph.p_va, ph.p_offset, ph.p_filesz, ph.p_memsz, offset);
            // copy TEXT/DATA section of bianry program
            kprintln!("TEXT/DATA");
            if offset > 0 {
                kprintln!("page offset: {}, binary offset: {}", offset, bin_off);
                let pa = match user_pgdir_alloc_page(&mut self.allocator, pgdir, va, perm) {
                    Ok(pa) => { pa as *mut u8 },
                    Err(_) => { return Err(-3); }
                };
                let size = PGSIZE - offset;
                memcpy( unsafe{ pa.add(offset) }, unsafe{ binary.add(bin_off) }, size);
                va += PGSIZE;
                bin_off += size;
            }
            let mut end = (ph.p_offset + ph.p_filesz) as usize;
            loop {
                if bin_off >= end { break; }
                kprintln!("page offset: {}, binary offset: {} va: {}", offset, bin_off, va);
                let pa = match user_pgdir_alloc_page(&mut self.allocator, pgdir, va, perm) {
                    Ok(pa) => { pa as *mut u8 },
                    Err(_) => { return Err(-3); }
                };
                let size = if bin_off + PGSIZE >= end {
                    PGSIZE
                } else {
                    end - bin_off
                };
                memcpy(pa, unsafe{ binary.add(bin_off) }, size);
                bin_off += PGSIZE;
                va += PGSIZE;
            }
            // build BSS section of binary program
            kprintln!("BSS");
            end = (ph.p_offset + ph.p_memsz) as usize;
            loop {
                if bin_off >= end { break; }
                let pa = match user_pgdir_alloc_page(&mut self.allocator, pgdir, va, perm) {
                    Ok(pa) => { pa as *mut u8 },
                    Err(_) => { return Err(-3); }
                };
                unsafe{ memset(pa, 0, PGSIZE); }
                va += PGSIZE;
                bin_off += PGSIZE;
            }
        }

        // build user stack memory
        user_pgdir_alloc_page(&mut self.allocator, pgdir, USTACKTOP-PGSIZE, perm).expect("user alloc page failed");
        user_pgdir_alloc_page(&mut self.allocator, pgdir, USTACKTOP-2*PGSIZE, perm).expect("user alloc page failed");
        user_pgdir_alloc_page(&mut self.allocator, pgdir, USTACKTOP-3*PGSIZE, perm).expect("user alloc page failed");
        user_pgdir_alloc_page(&mut self.allocator, pgdir, USTACKTOP-4*PGSIZE, perm).expect("user alloc page failed");


        self.trap_frame.ttbr0 = PADDR(pgdir as usize) as u64;
        kprintln!("ttbr0: {:x}", pgdir as usize);
        self.trap_frame.sp = USTACKTOP as u64;
        kprintln!("sp:    {:x}", USTACKTOP as usize);
        let pte = get_pte(pgdir as *const usize , 0 as usize, false).expect("get pte");
        kprintln!("pte    {:x}", unsafe{ *pte } );
        kprintln!("content:");
        let bits = unsafe { ptr::read(0x1811000 as *mut usize) };
        kprintln!("first instruction: {:x}", bits);
        kprintln!("end");
        kprintln!("============================================");
        Ok(0)
    }
}

