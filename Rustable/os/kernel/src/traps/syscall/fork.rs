use SCHEDULER;
use process::process::Process;
use process::state::State;
use allocator::page::{PADDR, KADDR};
use allocator::alloc_page;
use traps::TrapFrame;

use console::kprintln;

fn alloc_proc(father: &Process, tf: &mut TrapFrame) -> Process {
    let mut process = Process::new();
    process.trap_frame = Box::new(*tf);
    process.trap_frame.x0 = 0;
    
    process.state = State::Ready;
    process.parent = Some(father as *const Process);
    
    process.proc_name = String::from("child");
    
    let pgdir = KADDR(alloc_page().expect("alloc page for pgdir") as usize);
    process.trap_frame.ttbr0 = PADDR(pgdir) as u64;

    process.allocator.init_user(pgdir as *const usize);
    kprintln!("alloc_proc");
    process.allocator.copy_page(father.trap_frame.ttbr0 as *const usize, process.trap_frame.ttbr0 as *const usize);

    process
}

pub fn do_fork(tf: &mut TrapFrame) {
    kprintln!("fork");
    let current = SCHEDULER.pop_current();
    tf.x0 = SCHEDULER.last_id() + 1;
    kprintln!("father return value: {}", tf.x0);
    let process = alloc_proc(&current, tf);
    // process.parent = Some(Rc::new(current));
    
    // memcpy(pgidr as *mut u8, )
    // process.allocator = current.allocator.clone();
    // current.allocator.copy_page(current.trap_frame.ttbr0 as *const usize, process.trap_frame.ttbr0 as *const usize);
    // process.pid = get_unique_pid();
    

    SCHEDULER.push_current_front(current);
    SCHEDULER.add(process);
    kprintln!("fork finish");
}

