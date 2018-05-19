use process::Process;
use pi::timer::current_time;
use SCHEDULER;
use ALLOCATOR;

pub fn do_exec(ms: u32, tf: &mut TrapFrame) {
    let mut process = Process::new().unwrap();
    process.proc_init();
    process.trap_frame.ttbr0 = 0x01000000;
    // process.trap_frame.sp = process.stack.top().as_u64();
    process.trap_frame.elr = shell_thread as *mut u8 as u64;
    process.trap_frame.spsr = 0b000; // To EL 0, currently only unmasking IRQ
    load_icode(shell_thread as *mut u8, 0);
    
    if SCHEDULER.is_empty() {
        let tf = process.trap_frame.clone();
        BACKUP_ALLOCATOR = ALLOCATOR.switch_content(process.allocator);
        unsafe {
            asm!("mov sp, $0
              bl context_restore
              adr lr, _start
              mov sp, lr
              mov lr, xzr
              eret" :: "r"(tf) :: "volatile");
        };
    }

    let pending_pid = process.pid;

    let polling_fn = Box::new(move |process: &mut Process| {
        SCHEDULER.is_finished(pending_pid)
    });


    SCHEDULER.switch(State::Waiting(polling_fn), tf).unwrap();
}