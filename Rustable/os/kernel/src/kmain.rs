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
pub mod vm;
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

use process::sys_sleep;

pub extern "C" fn shell_thread() {
    // unsafe { asm!("brk 1" :::: "volatile"); }
    // shell::shell("$ ");
    // shell::shell("# ");
    // sys_sleep(1000);
    console::kprintln!("thread1");
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
    loop {
        shell::shell("# ");
        // aarch64::nop();
        // console::kprintln!("thread 2");
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

    let addr = &buf as *const alloc::Vec<i32> as *mut usize as usize;
    console::kprintln!("vec addr: {}", addr);

    let a = [1, 2, 3];

    assert_eq!(a.iter().find(|&&x| x == 2), Some(&2));

    assert_eq!(a.iter().find(|&&x| x == 5), None);

    FILE_SYSTEM.initialize();

    console::kprintln!("File system initialized!");

    // SCHEDULER.start();
    shell::shell("Rainable: ");
}
