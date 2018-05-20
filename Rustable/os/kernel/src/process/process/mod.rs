pub mod utils;

use traps::TrapFrame;
use process::State;
use std::string::String;

use allocator::imp::Allocator;
use allocator::util::{align_down};
use allocator::{alloc_page};
use mm::pmm::user_pgdir_alloc_page;
use allocator::page::{ Page, PGSIZE, USTACKTOP, PADDR, page2kva, page2va, ATTRIB_AP_RW_ALL, KADDR };

use self::utils::{memset, memcpy};
use process::elf::{Elfhdr, Proghdr};
// use console;
use std::mem;
use std;
use std::ptr;

use console::kprintln;

/// Type alias for the type of a process ID.
pub type Id = u64;


pub static mut pid: usize = 0;

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
        // create a new PDT, and mm->pgdir= kernel virtual addr of PDT
        let pgdir = match alloc_page() {
            Ok(paddr) => { KADDR(paddr as usize) as *const usize},
            Err(_) => { return Err(-1); }
        };

        self.allocator.init_user(pgdir);

        kprintln!("finish init user page");

        let elf = unsafe { ptr::read( binary as *const Elfhdr ) };
        // struct proghdr *ph = (struct proghdr *)(binary + elf->e_phoff);
        // let ph = unsafe { ptr::read( (&binary[0] as *mut u8).add(elf.phoff) as *const Proghdr ) };
        let phs = unsafe { std::slice::from_raw_parts_mut( binary.add(elf.e_phoff as usize) as *mut Proghdr, elf.e_phnum as usize) };
        // if elf.e_magic != ELF_MAGIC {
        //     return Err(-2);
        // }
        let mut ph_idx = 0;
        for ph in phs {
            kprintln!("ph idx: {}", ph_idx);
            ph_idx += 1;

            let mut offset = ph.p_va as usize - align_down(ph.p_va as usize, PGSIZE);
            let mut va = align_down(ph.p_va as usize, PGSIZE) as usize;
            let mut bin_off = ph.p_offset as usize;
            kprintln!("va: {:x} filesz: {} memsz: {} offset: {}", ph.p_va, ph.p_filesz, ph.p_memsz, offset);
            // copy TEXT/DATA section of bianry program
            kprintln!("TEXT/DATA");
            if offset > 0 {
                kprintln!("page offset: {}, binary offset: {}", offset, bin_off);
                let pa = match user_pgdir_alloc_page(&mut self.allocator, pgdir, va, ATTRIB_AP_RW_ALL) {
                    Ok(pa) => { pa as *mut u8 },
                    Err(_) => { return Err(-3); }
                };
                let size = PGSIZE - offset;
                memcpy( unsafe{ pa.add(offset) }, unsafe{ binary.add(bin_off) }, size);
                va += PGSIZE;
                bin_off += size;
            }
            let mut end = (ph.p_filesz) as usize;
            loop {
                if bin_off >= end { break; }
                kprintln!("page offset: {}, binary offset: {} va: {}", offset, bin_off, va);
                let pa = match user_pgdir_alloc_page(&mut self.allocator, pgdir, va, ATTRIB_AP_RW_ALL) {
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
            end = ph.p_memsz as usize;
            loop {
                if bin_off >= end { break; }
                let pa = match user_pgdir_alloc_page(&mut self.allocator, pgdir, va, ATTRIB_AP_RW_ALL) {
                    Ok(pa) => { pa as *mut u8 },
                    Err(_) => { return Err(-3); }
                };
                unsafe{ memset(pa, 0, PGSIZE); }
                va += PGSIZE;
                bin_off += PGSIZE;
            }
        }

        // build user stack memory
        user_pgdir_alloc_page(&mut self.allocator, pgdir, USTACKTOP-PGSIZE, ATTRIB_AP_RW_ALL).expect("user alloc page failed");
        user_pgdir_alloc_page(&mut self.allocator, pgdir, USTACKTOP-2*PGSIZE, ATTRIB_AP_RW_ALL).expect("user alloc page failed");
        user_pgdir_alloc_page(&mut self.allocator, pgdir, USTACKTOP-3*PGSIZE, ATTRIB_AP_RW_ALL).expect("user alloc page failed");
        user_pgdir_alloc_page(&mut self.allocator, pgdir, USTACKTOP-4*PGSIZE, ATTRIB_AP_RW_ALL).expect("user alloc page failed");

        self.trap_frame.ttbr0 = PADDR(pgdir as usize) as u64;
        self.trap_frame.sp = USTACKTOP as u64;
        Ok(0)
    }
}

