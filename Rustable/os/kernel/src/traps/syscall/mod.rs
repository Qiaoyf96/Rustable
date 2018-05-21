mod exec;
mod sleep;

use traps::TrapFrame;

use self::exec::do_exec;
use self::sleep::sleep;
use console::kprintln;

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
        3 => {
            use aarch64::get_ttbr0;
            let mut ttbr0 = unsafe { get_ttbr0() };
            kprintln!("ttbr: {:x}", ttbr0);
            kprintln!("user called!");
        }
        _ => {
            // x7 = 1, syscall does not exist.
            tf.x1to29[6] = 1;
        }
    }
}
