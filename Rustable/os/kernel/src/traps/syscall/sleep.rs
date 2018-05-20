

use process::Process;
use pi::timer::current_time;
use SCHEDULER;
use traps::trap_frame::TrapFrame;
use process::State;

pub fn sleep(ms: u32, tf: &mut TrapFrame) {
    let begin = current_time();
    let time = begin + ms as u64 * 1000;
    let polling_fn = Box::new(move |process: &mut Process| {
        let current = current_time();
        if current > time {
            process.trap_frame.x1to29[6] = 0; // x7 = 0; succeed
            process.trap_frame.x0 = (current - begin) / 1000; // x0 = elapsed time in ms
            true
        } else {
            false
        }
    });
    SCHEDULER.switch(State::Waiting(polling_fn), tf).unwrap();
}