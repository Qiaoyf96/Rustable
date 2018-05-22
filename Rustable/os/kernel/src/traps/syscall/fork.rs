use SCHEDULER;
use process::process::{Process};

fn alloc_proc() -> Process {
    let mut process = Process::new();
    // process.trap_frame = Box::new(TrapFrame::default());
    // process.state = State::Ready;
    // process.proc_name = "";
    process
}

pub fn do_fork() {
    // let mut current = SCHEDULER.pop_current();;

    // let mut process = alloc_proc();
    
    // process.parent = Box::(&current);

    // let pgdir = KADDR(alloc_page().expect("alloc page for pgdir") as usize);

    // memcpy(pgidr as *mut u8, )

    // process.allocator = current.allocator.clone();

    // process.trap_frame.ttbr0 = PADDR(pgdir) as *mut u8;

    // process.pid = get_unique_pid();

    // SCHEDULER.push_current_front(current);

    // SCHEDULER.add(process);
}

