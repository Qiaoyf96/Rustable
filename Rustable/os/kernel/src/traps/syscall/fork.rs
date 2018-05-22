use SCHEDULER;
use process::process::{Process, get_unique_pid};

fn alloc_proc() -> Process {
    let mut process = Process::new();
    process.trap_frame = Box::new(TrapFrame::default());
    process.state = State::;
    process.proc_name = "";
    process.allocator = None;
    process.pid = -1;
    process
}

fn do_fork() {
    let mut process = alloc_proc();
    
    process.parent = current;

    let pgdir = match alloc_page() {
        Ok(paddr) => { KADDR(paddr as usize) },
        Err(_) => { return Err(-1); }
    };

    process.allocator = SCHEDULTER.current.clone();

    process.trap_frame.ttbr0 = PADDR(pgdir) as *mut u8;

    process.pid = get_unique_pid();

    SCHEDULER.add(process);
}

