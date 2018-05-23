use SCHEDULER;
use allocator::dealloc_page;
use traps::trap_frame::TrapFrame;
use process::state::State;
use console::kprintln;

pub fn do_exit(tf: &mut TrapFrame) {
    kprintln!("exit");

    // let mut current = SCHEDULER.pop_current();

    // let pgdir = current.trap_frame.ttbr0;

    // dealloc_page(pgdir as *mut u8);

    // current.state = State::Zombie;

    // TODO: take the process out of the schedule list
    // SCHEDULER.push_current_front(current);

    SCHEDULER.switch(State::Zombie, tf).unwrap();
}

