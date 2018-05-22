use SCHEDULER;

fn do_exit(error_code: u32) {
    let pgdir = SCHEDULER.current_proc.trap_frame.ttbr0;
    dealloc_page(pgdir);

    SCHEDULER.current_proc.state = State::ZOMBIE;

    // TODO: take the process out of the schedule list

    // TODO: schedule = a
}

