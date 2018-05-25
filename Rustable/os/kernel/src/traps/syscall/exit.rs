use SCHEDULER;
use traps::trap_frame::TrapFrame;
use process::state::State;
use console::kprintln;
use shell;

pub fn do_exit(tf: &mut TrapFrame) {
    kprintln!("exit");

    let mut current = SCHEDULER.pop_current();

    let pgdir = current.trap_frame.ttbr0;
    current.allocator.clear_page(pgdir as *const usize);
    SCHEDULER.push_current_front(current);

    if SCHEDULER.switch(State::Zombie, tf) == None {
        SCHEDULER.clear();
        // kprintln!("enter shell");
        shell::shell("Rainable: ");
    }
}

