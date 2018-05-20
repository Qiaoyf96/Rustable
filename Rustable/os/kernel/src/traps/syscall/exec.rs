use process::Process;
use SCHEDULER;
use ALLOCATOR;
use shell_thread;
use allocator::imp::BACKUP_ALLOCATOR;
use traps::trap_frame::TrapFrame;
use process::State;
use allocator::imp::Allocator;
use std::mem;

pub fn do_exec(ms: u32, tf: &mut TrapFrame) {
    let mut process = Process::new();
    process.proc_init();
    process.trap_frame.ttbr0 = 0x01000000;
    // process.trap_frame.sp = process.stack.top().as_u64();
    process.trap_frame.elr = shell_thread as *mut u8 as u64;
    process.trap_frame.spsr = 0b000; // To EL 0, currently only unmasking IRQ
    process.load_icode(shell_thread as *mut u8, 0);
    
    if SCHEDULER.is_empty() {
        let tf = process.trap_frame.clone();
        ALLOCATOR.switch_content(&process.allocator as *const Allocator as *mut Allocator, unsafe { mem::transmute(BACKUP_ALLOCATOR) });
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