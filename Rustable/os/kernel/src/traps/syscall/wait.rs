use traps::trap_frame::TrapFrame;
use SCHEDULER;
use process::process::Process;
use process::state::State;
use console::kprintln;

pub fn do_wait(id: u32, tf: &mut TrapFrame) {
    kprintln!("wait {}", id);
    // let waiting_fn = Box::new(move |process: &mut Process| {
    //     if let State::Zombie = process.state {
    //         if process.trap_frame.tpidr == id as u64 {
    //             true
    //         }
    //         else {
    //             false
    //         }
    //     } else {
    //         false
    //     }
    // });
    SCHEDULER.switch(State::Wait_Proc(id), tf).unwrap();
}