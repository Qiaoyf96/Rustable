use SCHEDULER;
use process::process::{Process, get_unique_pid};
use process::state::State;
use allocator::page::{PADDR, KADDR};
use allocator::alloc_page;

fn alloc_proc(father: &Process) -> Process {
    let mut process = Process::new();
    process.trap_frame = Box::new(*father.trap_frame.as_ref());
    process.state = State::Ready;
    process.parent = Some(father as *const Process);
    
    process.proc_name = String::from("child");
    
    let pgdir = KADDR(alloc_page().expect("alloc page for pgdir") as usize);
    process.trap_frame.ttbr0 = PADDR(pgdir) as u64;
    process.pid = get_unique_pid();

    process.allocator.init_user(pgdir as *const usize);
    process.allocator.copy_page(father.trap_frame.ttbr0 as *const usize, process.trap_frame.ttbr0 as *const usize);

    process
}

pub fn do_fork() {
    let current = SCHEDULER.pop_current();;
    let process = alloc_proc(&current);
    // process.parent = Some(Rc::new(current));
    
    // memcpy(pgidr as *mut u8, )
    // process.allocator = current.allocator.clone();
    // current.allocator.copy_page(current.trap_frame.ttbr0 as *const usize, process.trap_frame.ttbr0 as *const usize);
    // process.pid = get_unique_pid();
    
    SCHEDULER.push_current_front(current);
    SCHEDULER.add(process);
}

