#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(exclusive_range_pattern)]
#![feature(i128_type)]
#![feature(never_type)]
#![feature(unique)]
#![feature(pointer_methods)]
#![feature(naked_functions)]
#![feature(fn_must_use)]
#![feature(alloc, allocator_api, global_allocator)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;
extern crate pi;
extern crate stack_vec;
extern crate fat32;

pub mod allocator;
pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;
pub mod traps;
pub mod aarch64;
pub mod process;
pub mod fs;
pub mod mm;

#[cfg(not(test))]
use allocator::Allocator;
use fs::FileSystem;
use mm::pmm::Pmm;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();

pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();

#[cfg(not(test))]
use process::GlobalScheduler;
use pi::timer::{spin_sleep_ms};

use process::syscall::sys_sleep;
use shell::copy_elf;

pub extern "C" fn shell_thread() {
    // unsafe { console::kprintln!("pc: {:x}", aarch64::get_pc()); }
    // shell::shell("$ ");
    // shell::shell("# ");
    // sys_sleep(1000);
    unsafe { asm!("svc 2" :::: "volatile"); }
    unsafe { asm!("svc 2" :::: "volatile"); }
    unsafe { asm!("svc 2" :::: "volatile"); }
    unsafe { asm!("svc 2" :::: "volatile"); }
    unsafe { asm!("svc 2" :::: "volatile"); }
    // sys_sleep(1000);
    console::kprintln!("thread1");
    
    let illegal_addr: usize = 8;
    let illegal_val = unsafe { *(illegal_addr as *const usize) };
    console::kprintln!("try to access illegal addr {:x}: {}", illegal_addr, illegal_val);

    loop {
        sys_sleep(1000);
        console::kprintln!("thread1");
        // aarch64::nop();
        // console::kprintln!("thread 1");
        // shell::shell("$ ");
    }
}

pub extern "C" fn shell_thread_2() {
    console::kprintln!("thread2");

    // let illegal_addr: usize = 8;
    // let illegal_val = unsafe { *(illegal_addr as *const usize) };
    // console::kprintln!("try to access illegal addr {:x}: {}", illegal_addr, illegal_val);

    loop {
        // shell::shell("# ");
        // aarch64::nop();
        sys_sleep(1000);
        console::kprintln!("thread 2");
    }
}

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn kmain() {
    // FIXME: Start the shell.
    // ALLOCATOR.initialize();
    spin_sleep_ms(1000);
    let begin = r#"
            @@@@@@@@@@                                                                                                   
       *@@@     ,   @@@                                       ,                     @@@@@          @@@@@                
     -@@@  =@@@@    @@@@                                  @@@@@                     @@@@#         =@@@@                 
    ;@@@   @@@@@    @@@                                  :@@@@                     @@@@@          @@@@@                 
    @@@    @@@@   ;@@@    @@@@@    @@@@@      @@@@@     @@@@@@@@     @@@@@@@@@@@   @@@@@@@@@@~    @@@@      @@@@@@@~    
     @@@  @@@@@@@@@@$     @@@@-    @@@@      @@@@@@      @@@@,     @@@@   $@@@@   @@@@@    @@@@  @@@@@    @@@@@   @@    
          @@@@$  -@@@@   @@@@@    @@@@@      @ @@@@#    @@@@@    @@@@@    @@@@@   @@@@@    @@@@  @@@@#   @@@@@    @@    
         @@@@@    @@@@   @@@@@    @@@@!     @  @@@@@    @@@@@    @@@@     @@@@   .@@@@     @@@$ @@@@@    @@@@   #@@     
         @@@@@   @@@@@  @@@@@    @@@@@    @@@@  @@@@@  @@@@@    @@@@@    @@@@@   @@@@@    @@@@  @@@@@   @@@@@@@@        
        #@@@@    @@@@   @@@@@    @@@@@   @@@@@  @@@@@-@@@@@@    @@@@     @@@@#   @@@@*    @@@  #@@@@    @@@@#           
        @@@@@   @@@@@   @@@@@  ,@@@@@@  ~@@     @@@@@  @@@@@  ,@@@@@@   @@@@@=  @@@@@   ~@@@   @@@@@   @@@@@@     @     
       .@@@@    @@@@@   @@@@@@@  @@@@@@@  @@@@@@@@@    @@@@@@@  ;@@@@@@@ @@@@@@@ @@@@@@@@@.     @@@@@@@  @@@@@@@@@      
                 .#=                                                                                          "#;
    console::kprint!("{}\n", begin);
    
    let pmm = Pmm;
    pmm.init();
    
    console::kprintln!("Physical memory initialized!");

    // DEBUG
    // let mut buf = vec![];
    let buf = vec![1, 2, 3, 4, 5];
    console::kprintln!("vec test!");
    // buf.push(1);
    console::kprintln!("vec test: {} {} {} {} {} {} ", buf.len(), buf[0], buf[1], buf[2], buf[3], buf[4]);

    let addr = &buf[0] as *const i32 as *mut usize as usize;
    console::kprintln!("vec addr: {:x}", addr);

    let a = [1, 2, 3];

    assert_eq!(a.iter().find(|&&x| x == 2), Some(&2));

    assert_eq!(a.iter().find(|&&x| x == 5), None);

    FILE_SYSTEM.initialize();

    console::kprintln!("File system initialized!");


    // let illegal_addr: usize = 512*1024*1024 * 2+8;
    // let illegal_val = unsafe { *(illegal_addr as *const usize) };
    use mm::pmm::{page_remove};
    use mm::vm::{get_pte};
    use aarch64::get_ttbr0;
    let ttbr0 = unsafe { get_ttbr0() };
    console::kprintln!("ttbr: {:x}", ttbr0);
    
    copy_elf();
    // unsafe { asm!("svc 3" :::: "volatile"); }
    // page_remove(ttbr0 as *const usize, 0x15c1000, get_pte(ttbr0 as *const usize , 0x15c1000, false).expect(""));
    // let illegal_addr: usize = 0x14c1008;
    
    // let illegal_val = unsafe { *(illegal_addr as *const usize) };
    // console::kprintln!("try to access illegal addr {:x}: {}", illegal_addr, illegal_val);
    // let illegal_val = unsafe { *(illegal_addr as *const usize) };
    // console::kprintln!("try to access illegal addr {:x}: {}", illegal_addr, illegal_val);
    
    
    // console::kprintln!("===schedule===");
    SCHEDULER.start();
    // shell::shell("Rainable: ");
    // console::kprintln!("========================end===========================");
    // loop {
    //     // shell::shell("# ");
    //     aarch64::nop();
    //     // sys_sleep(1000);
    //     // console::kprintln!("thread 2");
    // }
}
