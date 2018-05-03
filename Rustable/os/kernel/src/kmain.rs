#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(exclusive_range_pattern)]
#![feature(alloc, allocator_api, global_allocator)]
#![feature(pointer_methods)]

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
pub mod fs;

#[cfg(not(test))]
use allocator::Allocator;
use fs::FileSystem;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();

use pi::timer::{spin_sleep_ms, current_time};



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
    ALLOCATOR.initialize();
    FILE_SYSTEM.initialize();
    
    shell::shell("Rainable: ");
    
}
