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

pub mod allocator;
pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;
pub mod traps;
pub mod aarch64;
pub mod process;
pub mod vm;

#[cfg(not(test))]
use process::GlobalScheduler;
use pi::timer::{spin_sleep_ms, current_time};

#[cfg(not(test))]
use allocator::Allocator;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();
// pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();

#[no_mangle]
pub extern "C" fn kmain() {
    // FIXME: Start the shell.
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
    console::kprint!("{}\n", unsafe {aarch64::current_el()});

    unsafe { asm!("brk 2" :::: "volatile"); }
    
    shell::shell("Rainable: ");
}
