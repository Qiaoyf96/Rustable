use process::Process;
use SCHEDULER;
use ALLOCATOR;
use shell_thread;
use allocator::imp::BACKUP_ALLOCATOR;
use traps::trap_frame::TrapFrame;
use process::State;
use allocator::imp::Allocator;
use std::mem;
use console::kprintln;
use std::ptr;
use aarch64::get_ttbr0;
use mm::vm::get_pte;

pub fn do_exec(ms: u32, tf: &mut TrapFrame) {
    let mut process = Process::new();
    process.proc_init();
    process.trap_frame.ttbr0 = 0x01000000;
    // process.trap_frame.sp = process.stack.top().as_u64();
    process.trap_frame.elr = (0x0) as *mut u8 as u64;
    process.trap_frame.spsr = 0b000; // To EL 0, currently only unmasking IRQ
    process.load_icode((0x14c7000)  as *mut u8, 0);
    
    if SCHEDULER.is_empty() {
        kprintln!("tf ttbr0 {:x}", process.trap_frame.ttbr0);
        let tf = process.trap_frame.clone();
        kprintln!("tf ttbr0: {:x}", tf.ttbr0);
        ALLOCATOR.switch_content(&process.allocator as *const Allocator as *mut Allocator, unsafe { mem::transmute(BACKUP_ALLOCATOR) });
        let mut ttbr0 = unsafe { get_ttbr0() };
        kprintln!("ttbr: {:x}", ttbr0);
        kprintln!("ins: {:x}", unsafe { ptr::read(0 as *mut u32) });
        let mut pte = get_pte(ttbr0 as *const usize , 0 as usize, false).expect("get pte");
        kprintln!("pte   {:x}", unsafe{ *pte } );
        unsafe {
            asm!("ldr lr, =0x14cb000
              msr ttbr0_el1, lr
              isb":::::"volatile");
        };
        kprintln!("gap----------------------------");
        let mut ttbr0 = unsafe { get_ttbr0() };
        kprintln!("ins: {:x}", unsafe { ptr::read(0 as *mut u32) });
        pte = get_pte(ttbr0 as *const usize , 0 as usize, false).expect("get pte");
        kprintln!("pte   {:x}", unsafe{ *pte } );
        kprintln!("ttbr: {:x}", ttbr0);
        
        unsafe {
            asm!("ldr lr, =0x1000000
              msr ttbr0_el1, lr
              isb"::::"volatile");
        };
        kprintln!("gap----------------------------");
        ttbr0 = unsafe { get_ttbr0() };
        kprintln!("ttbr: {:x}", ttbr0);
        ttbr0 = unsafe { get_ttbr0() };
        kprintln!("ttbr: {:x}", ttbr0);
        ttbr0 = unsafe { get_ttbr0() };
        kprintln!("ttbr: {:x}", ttbr0);
        ttbr0 = unsafe { get_ttbr0() };
        kprintln!("ttbr: {:x}", ttbr0);
        ttbr0 = unsafe { get_ttbr0() };
        kprintln!("ttbr: {:x}", ttbr0);

        kprintln!("ins: {:x}", unsafe { ptr::read(0 as *mut u32) });
        pte = get_pte(ttbr0 as *const usize , 0 as usize, false).expect("get pte");
        kprintln!("pte   {:x}", unsafe{ *pte } );
    }

    let pending_pid = process.pid;

    let polling_fn = Box::new(move |process: &mut Process| {
        SCHEDULER.is_finished(pending_pid)
    });


    SCHEDULER.switch(State::Waiting(polling_fn), tf).unwrap();
}