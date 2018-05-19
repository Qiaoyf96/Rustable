use traps::TrapFrame;
use process::State;
use process::Process;
use pi::timer::current_time;
use SCHEDULER;
// use console;

/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.


pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    match num {
        1 => {
            sleep(tf.x0 as u32, tf);
        },
        2 => {
            do_exec(tf.x0 as u32, tf);
        }
        _ => {
            // x7 = 1, syscall does not exist.
            tf.x1to29[6] = 1;
        }
    }
}
