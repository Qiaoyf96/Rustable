use ALLOCATOR;
mod irq;
mod trap_frame;
pub mod syndrome;
mod syscall;

use pi::interrupt::{Controller, Interrupt};

use mm::vm::page_fault::do_pgfault;

pub use self::trap_frame::TrapFrame;

// use console::kprintln;
// use aarch64;
use self::syndrome::Syndrome;
use self::irq::handle_irq;
use self::syscall::handle_syscall;
use allocator::imp::{ USER_ALLOCATOR, BACKUP_ALLOCATOR, Allocator };
use console::kprintln;
use std::mem;
use aarch64::get_ttbr0;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}

/// This function is called when an exception occurs. The `info` parameter
/// specifies the source and kind of exception that has occurred. The `esr` is
/// the value of the exception syndrome register. Finally, `tf` is a pointer to
/// the trap frame for the exception.
#[no_mangle]
pub extern fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    kprintln!("{:?} {:?} {:b}", info.source, info.kind, esr);
    unsafe { ALLOCATOR.switch_content(mem::transmute(BACKUP_ALLOCATOR), mem::transmute(USER_ALLOCATOR)); }
    kprintln!("finish switch allocator");
    // kprintln!("{:?} {:?}", info.kind, info.kind==Kind::SError);
    if info.kind == Kind::Synchronous {
        // kprintln!("syn");
        match Syndrome::from(esr) {
            Syndrome::Brk(i) => {
                // shell::shell(" [brk]$ ");
                kprintln!("brk");
                tf.elr += 4;
                unsafe { ALLOCATOR.switch_content(mem::transmute(USER_ALLOCATOR), mem::transmute(BACKUP_ALLOCATOR)); }
                return;
            },
            Syndrome::Svc(syscall) => {
                kprintln!("syscall");
                handle_syscall(syscall, tf);
                unsafe { ALLOCATOR.switch_content(mem::transmute(USER_ALLOCATOR), mem::transmute(BACKUP_ALLOCATOR)); }
                return;
            },
            Syndrome::InstructionAbort{kind, level} => {
                kprintln!("InstructionAbort");

                kprintln!("ttbr0: {:x}", unsafe { get_ttbr0() });

                // do_pgfault(kind, level);
                unsafe { ALLOCATOR.switch_content(mem::transmute(USER_ALLOCATOR), mem::transmute(BACKUP_ALLOCATOR)); }
                return;
            },
            Syndrome::DataAbort{kind, level} => {
                kprintln!("DataAbort");
                do_pgfault(kind, level);
                unsafe { ALLOCATOR.switch_content(mem::transmute(USER_ALLOCATOR), mem::transmute(BACKUP_ALLOCATOR)); }
                return;
            },
            _ => { kprintln!{"unknown type"}; }
        }
    } else if info.kind == Kind::Irq {
        let controller = Controller::new();
        use self::Interrupt::*;
        for interrupt in [Timer1, Timer3, Usb, Gpio0, Gpio1, Gpio2, Gpio3, Uart].iter() {
            if controller.is_pending(*interrupt) {
                handle_irq(*interrupt, tf);
                unsafe { ALLOCATOR.switch_content(mem::transmute(USER_ALLOCATOR), mem::transmute(BACKUP_ALLOCATOR)); }
                return;
            }
        }
    }
    kprintln!("halt");
    loop {
        unsafe { asm!("wfe") }
    }
}
