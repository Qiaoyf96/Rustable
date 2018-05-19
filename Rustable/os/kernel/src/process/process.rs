use traps::TrapFrame;
use process::{State, Stack};

// use console;
use std::mem;

/// Type alias for the type of a process ID.
pub type Id = u64;

/// A structure that represents the complete state of a process.
#[derive(Debug)]
pub struct Process {
    /// The saved trap frame of a process.
    pub trap_frame: Box<TrapFrame>,
    /// The memory allocation used for the process's stack.
    pub stack: Stack,
    /// The scheduling state of the process.
    pub state: State,
}

impl Process {
    /// Creates a new process with a zeroed `TrapFrame` (the default), a zeroed
    /// stack of the default size, and a state of `Ready`.
    ///
    /// If enough memory could not be allocated to start the process, returns
    /// `None`. Otherwise returns `Some` of the new `Process`.
    pub fn new() -> Option<Process> {
        match Stack::new() {
            Some(stack) => Some(Process {
                trap_frame: Box::new(TrapFrame::default()),
                stack,
                state: State::Ready,
            }),
            None => None,
        }
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

fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.offset(i as isize) = *src.offset(i as isize);
        i += 1;
    }
    return dest;
}

pub unsafe extern fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.offset(i as isize) = c as u8;
        i += 1;
    }
    return s;
}


fn load_icode(binary: &mut [u8], size: usize) -> Result<i32, i32> {
    // create a new PDT, and mm->pgdir= kernel virtual addr of PDT
    let pgdir = match alloc_page() {
        Ok(page) => { page2kva(page as *const Page) },
        Err(_) => { return Err(-1); }
    }

    let elf = unsafe { ptr::read( (&binary[0]) as *const u8 as *const Elfhdr ) };
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
            let page = match pgdir_alloc_page(process.pgdir, offset, ATTRIB_AP_RW_ALL) {
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
            let page = match pgdir_alloc_page(process.pgdir, va, ATTRIB_AP_RW_ALL) {
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
            let page = match pgdir_alloc_page(process.pgdir, va, ATTRIB_AP_RW_ALL) {
                Ok(page) => { page2kva(page as *mut Page) },
                Err(_) => { return Err(-3) }
            };
            memset(page as *mut u8, 0, PGSIZE);
            va += PGSIZE
            bin_off += PGSIZE;
        }
    }

    // set trapframe
    
}
