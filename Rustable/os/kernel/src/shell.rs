use stack_vec::StackVec;
use console::{kprint, kprintln, CONSOLE};
use std;
use std::path::{Path, PathBuf};
use pi;
use FILE_SYSTEM;
use fat32::traits::{Dir, Entry, FileSystem, Timestamp, Metadata};
use allocator::alloc_page;
use SCHEDULER;
use mutex::Mutex;
use PWD;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs
}


// pub static PWD: Mutex<Option<String>> = Mutex::new(None);
pub struct Pwd(Mutex<Option<String>>);

impl Pwd {
    pub const fn uninitialized() -> Self {
        Pwd(Mutex::new(None))
    }

    pub fn initialize(&self) {
        *self.0.lock() = Some(String::from("/"));
    }

    pub fn get_string(&self) -> String {
        self.0.lock().clone().unwrap()
    }
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

const LF: u8 = b'\n';
const CR: u8 = b'\r';
const BACK: u8 = 8u8;
const DEL: u8 = 127u8;
const BELL: u8 = 7u8;
const SPACE: u8 = b' ';

fn read_line<'a>(buf_vec: &'a mut StackVec<'a, u8>) -> &'a str {
    loop {
        let single_byte = {
            let mut console = CONSOLE.lock();
            console.read_byte()
        };
        // kprintln!("single byte {}", single_byte);
        match single_byte {
            CR => break,
            BACK | DEL => {
                match buf_vec.pop() {
                    Some(_) => {
                        let mut console = CONSOLE.lock();
                        console.write_byte(BACK);
                        console.write_byte(b' ');
                        console.write_byte(BACK);
                    },
                    None => { 
                        let mut console = CONSOLE.lock();
                        console.write_byte(BELL); 
                    }
                }
            },
            byte if byte.is_ascii_graphic() || byte == SPACE => {
                match buf_vec.push(byte) {
                    Ok(_) => {
                        let mut console = CONSOLE.lock();
                        console.write_byte(byte);
                    },
                    Err(_) => {
                        let mut console = CONSOLE.lock();
                        console.write_byte(BELL);
                    }
                }
            }
            _ => {
                let mut console = CONSOLE.lock();
                console.write_byte(BELL);
            }
        }
    }
    {
        let mut console = CONSOLE.lock();
        console.write_byte(b'\r');
        console.write_byte(b'\n');
    }
    std::str::from_utf8(buf_vec.as_slice()).unwrap_or("")
}

pub fn copy_elf(file: &str) -> usize {
    let mut working_dir = PathBuf::from("/");
    handle_cpy(file, &mut working_dir)
}


/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns: it is perpetually in a shell loop.
pub fn shell(prefix: &str) -> ! {
    let mut working_dir = PathBuf::from(PWD.get_string());
    // kprint!("pwd: {}", PWD.get_string());
    let mut i = 0;
    loop {
        // kprintln!("i={}",i);
        i += 1;
        kprint!("$ {}", prefix);
        let mut buf_vec = [0u8; 512];
        let mut inputs = StackVec::new(&mut buf_vec);
        let input_line = read_line(&mut inputs);
        // kprintln!("read line {}", input_line);
        let mut buf = [""; 64];
        match Command::parse(input_line, &mut buf) {
            Ok(ref command) => {
                match command.path() {
                    "echo" => echo(&command.args[1..]),
                    "atag" => print_atags(),
                    "pwd" => handle_pwd(&command.args[1..], &mut working_dir),
                    "cd" => handle_cd(&command.args[1..], &mut working_dir),
                    "ls" => handle_ls(&command.args[1..], &mut working_dir),
                    "cat" => handle_cat(&command.args[1..], &mut working_dir),
                    "exec" => handle_exec(&command.args[1..], &mut working_dir),
                    // "cpy" => handle_cpy(&command.args[1..], &mut working_dir),
                    // "v" => handle_v(),
                    "exit" => exit(),
                    unknown => {
                        kprint!("unknown command: {}\n", unknown);
                    }
                }
            }
            Err(err) => {
                match err {
                    Error::TooManyArgs => kprint!("error: too many arguments\n"),
                    _ => { },
                }
            }
        }
    }
}

fn echo(args: &[&str]) {
    for arg in args.iter() {
        kprint!("{} ", arg);
    }
    kprint!("\n");
}

fn print_atags() {
    for atag in pi::atags::Atags::get() {
        kprintln!("{:#?}", atag);
    }
}

fn jump_to(addr: *mut u8) -> ! {
    unsafe {
        asm!("br $0" : : "r"(addr as usize));
        loop { asm!("nop" :::: "volatile")  }
    }
}

fn handle_pwd(args: &[&str], working_dir: &PathBuf) {
    if args.len() > 0 {
        kprintln!("Too many args. Usage:");
        kprintln!("pwd");
        kprintln!();
        return;
    }

    kprintln!("{}", working_dir.as_path().display());
}

fn handle_cd(args: &[&str], working_dir: &mut PathBuf) {
    if args.len() != 1 {
        kprintln!("Usage:");
        kprintln!("cd <directory>");
        kprintln!();
        return;
    }

    if args[0] == "." {
        // No-op.
    } else if args[0] == ".." {
        working_dir.pop();
    } else {
        let path = Path::new(args[0]);

        let mut new_dir = working_dir.clone();
        new_dir.push(path);

        let entry = FILE_SYSTEM.open(new_dir.as_path());
        if entry.is_err() {
            kprintln!("Path not found.");
            return;
        }
        if entry.unwrap().as_dir().is_some() {
            working_dir.push(path);
        } else {
            kprintln!("Not a directory.");
        }
    }
    *PWD.0.lock() = Some(working_dir.to_str().unwrap().to_string());
}

fn recover_pwd() {

}

fn print_entry<E: Entry>(entry: &E) {
    fn write_bool(b: bool, c: char) {
        if b { kprint!("{}", c); } else { kprint!("-"); }
    }

    fn write_timestamp<T: Timestamp>(ts: T) {
        kprint!("{:02}/{:02}/{} {:02}:{:02}:{:02} ",
               ts.month(), ts.day(), ts.year(), ts.hour(), ts.minute(), ts.second());
    }

    write_bool(entry.is_dir(), 'd');
    write_bool(entry.is_file(), 'f');
    write_bool(entry.metadata().read_only(), 'r');
    write_bool(entry.metadata().hidden(), 'h');
    kprint!("\t");

    write_timestamp(entry.metadata().created());
    write_timestamp(entry.metadata().modified());
    write_timestamp(entry.metadata().accessed());
    kprint!("\t");

    kprintln!("{}", entry.name());
}

fn handle_ls(mut args: &[&str], working_dir: &PathBuf) {
    let show_hidden = args.len() > 0 && args[0] == "-a";
    if show_hidden {
        args = &args[1..];
    }
    // kprintln!("1");
    if args.len() > 1 {
        kprintln!("Usage:");
        kprintln!("ls [-a] [directory]");
        kprintln!();
        return;
    }
    // kprintln!("2");
    let mut dir = working_dir.clone();
    // kprintln!("3");
    if !args.is_empty() {
        if args[0] == "." {
            // No-op.
        } else if args[0] == ".." {
            dir.pop();
        } else {
            dir.push(args[0]);
        }
    }
    // kprintln!("4");
    // use std::path::Display;
    // kprintln!("{}", dir.as_path().display());
    let entry_result = FILE_SYSTEM.open(dir.as_path());
    // kprintln!("5");
    if entry_result.is_err() {
        kprintln!("Path not found.");
        return;
    }
    // kprintln!("6");
    let entry = entry_result.unwrap();
    // kprintln!("7");
    if let Some(dir_entry) = entry.into_dir() {
        let mut entries = dir_entry.entries().expect("List dir");
        for item in entries {
            if show_hidden || !item.metadata().hidden() {
                print_entry(&item);
            }
        }
    } else {
        kprintln!("Not a directory.");
    }
    // kprintln!("8");
}

fn handle_cat(args: &[&str], working_dir: &PathBuf) {
    // kprintln!("cat");
    if args.len() != 1 {
        kprintln!("Usage:");
        kprintln!("cat <file>");
        kprintln!();
        return;
    }

    // kprintln!("cat");
    let mut dir = working_dir.clone();
    dir.push(args[0]);

    // kprintln!("cat-b");
    let entry_result = FILE_SYSTEM.open(dir.as_path());
    // kprintln!("cat-a");
    if entry_result.is_err() {
        kprintln!("Path not found.");
        return;
    }

    let entry = entry_result.unwrap();
    if let Some(ref mut file) = entry.into_file() {
        loop {
            use std::io::Read;

            let mut buffer = [0u8; 512];
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(_) => kprint!("{}", String::from_utf8_lossy(&buffer)),
                Err(e) => kprint!("Failed to read file: {:?}", e)
            }
        }

        kprintln!("");
    } else {
        kprintln!("Not a file.");
    }
}

const BOOTLOADER_START_ADDR: usize = 0x4000000;

fn exit() {
    kprintln!("You will exit to write a new kernel");
    jump_to(BOOTLOADER_START_ADDR as *mut u8);
}


fn handle_cpy(args: &str, working_dir: &PathBuf) -> usize {
    kprintln!("cpy");

    let mut dir = working_dir.clone();
    dir.push(args);

    let entry_result = FILE_SYSTEM.open(dir.as_path());

    if entry_result.is_err() {
        kprintln!("Path not found.");
        return 0;
    }

    let mut pa = alloc_page().expect("alloc pages failed");
    let elf_addr = pa as usize;
    kprint!("Elf {} addr: {:x}", args, pa as usize);
    let entry = entry_result.unwrap();
    if let Some(ref mut file) = entry.into_file() {
        loop {
            use std::io::Read;

            let mut buffer = [0u8; 4096];
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(_) => {
                    kprint!("{}", String::from_utf8_lossy(&buffer));
                    memcpy(pa, &buffer,4096);
                    pa = unsafe{ pa.add(4096) };

                },
                Err(e) => kprint!("Failed to read file: {:?}", e)

            }
        }

        kprintln!("");
        return elf_addr;
    } else {
        kprintln!("Not a file.");
    }
    0
}

fn handle_exec(args: &[&str], working_dir: &PathBuf) {
    // kprintln!("exec");

    let mut dir = working_dir.clone();
    dir.push(args[0]);

    let entry_result = FILE_SYSTEM.open(dir.as_path());

    if entry_result.is_err() {
        kprintln!("Path not found.");
        return;
    }

    let mut pa = alloc_page().expect("alloc pages failed");
    let elf_addr = pa as usize;

    let entry = entry_result.unwrap();
    if let Some(ref mut file) = entry.into_file() {
        loop {
            use std::io::Read;

            let mut buffer = [0u8; 4096];
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(_) => {
                    // kprint!("{}", String::from_utf8_lossy(&buffer));
                    memcpy(pa, &buffer,4096);
                    pa = unsafe{ pa.add(4096) };

                },
                Err(e) => kprint!("Failed to read file: {:?}", e)

            }
        }
        SCHEDULER.start(elf_addr);
        // kprintln!("");
        // unsafe { 
        //     asm!("mov x0, $2
        //         svc 2"
        //         :: "r"(elf_addr)
        //         : "x0", "x7":"volatile"
        //     ); 
        // }
    } else {
        kprintln!("Not a file.");
    }
}

fn memcpy(dest: *mut u8, buf: &[u8], n: usize) {
    let mut i = 0;
    while i < n {
        unsafe { *dest.offset(i as isize) = buf[i]; }
        i += 1;
    }
}


fn handle_v() {
    let pa = 0x150f000;
    let bits = unsafe { std::slice::from_raw_parts_mut(pa as *mut u8, 100) };
    kprint!("{}", String::from_utf8_lossy(&bits));
    kprintln!("");
    let bin = unsafe{ std::slice::from_raw_parts_mut( pa as *mut u32, 20) };
    kprintln!("~~~ cpy instruction ~~~");
    for ins in bin {
        kprintln!("{:b}", ins);
    }
    kprintln!("~~~~~~~~~~~~~~~~~~~");
}