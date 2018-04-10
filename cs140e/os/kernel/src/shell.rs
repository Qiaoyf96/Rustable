use stack_vec::StackVec;
use console::{kprint, kprintln, CONSOLE};
use std;
/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs
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
    let mut console = CONSOLE.lock();
    loop {
        match console.read_byte() {
            LF | CR => break,
            BACK | DEL => {
                match buf_vec.pop() {
                    Some(_) => {
                        console.write_byte(BACK);
                        console.write_byte(b' ');
                        console.write_byte(BACK);
                    },
                    None => { console.write_byte(BELL); }
                }
            },
            byte if byte.is_ascii_graphic() || byte == SPACE => {
                match buf_vec.push(byte) {
                    Ok(_) => console.write_byte(byte),
                    Err(_) => console.write_byte(BELL)
                }
            }
            _ => console.write_byte(BELL)
        }
    }
    console.write_byte(b'\r');
    console.write_byte(b'\n');
    std::str::from_utf8(buf_vec.as_slice()).unwrap_or("")
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns: it is perpetually in a shell loop.
pub fn shell(prefix: &str) -> ! {
    loop {
        kprint!("{}", prefix);
        let mut buf_vec = [0u8; 512];
        let mut inputs = StackVec::new(&mut buf_vec);
        let input_line = read_line(&mut inputs);
        let mut buf = [""; 64];
        match Command::parse(input_line, &mut buf) {
            Ok(ref command) => {
                match command.path() {
                    "echo" => echo(command),
                    unknown => {
                        kprint!("unknown command: {}\n", unknown);
                    }
                }
            },
            Err(err) => {
                match err {
                    Error::TooManyArgs => kprint!("error: too many arguments\n"),
                    _ => { },
                }
                
            }
        }
    }
}

fn echo(command: &Command) {
    for arg in command.args[1..].iter() {
        kprint!("{} ", arg);
    }
    kprint!("\n");
}