extern crate serial;
extern crate structopt;
extern crate xmodem;
#[macro_use] extern crate structopt_derive;

use std::path::PathBuf;
use std::time::Duration;

use structopt::StructOpt;
use serial::core::{CharSize, BaudRate, StopBits, FlowControl, SerialDevice, SerialPortSettings};
use xmodem::{Xmodem, Progress};

mod parsers;

use parsers::{parse_width, parse_stop_bits, parse_flow_control, parse_baud_rate};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(short = "i", help = "Input file (defaults to stdin if not set)", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short = "b", long = "baud", parse(try_from_str = "parse_baud_rate"),
                help = "Set baud rate", default_value = "115200")]
    baud_rate: BaudRate,

    #[structopt(short = "t", long = "timeout", parse(try_from_str),
                help = "Set timeout in seconds", default_value = "10")]
    timeout: u64,

    #[structopt(short = "w", long = "width", parse(try_from_str = "parse_width"),
                help = "Set data character width in bits", default_value = "8")]
    char_width: CharSize,

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(short = "f", long = "flow-control", parse(try_from_str = "parse_flow_control"),
                help = "Enable flow control ('hardware' or 'software')", default_value = "none")]
    flow_control: FlowControl,

    #[structopt(short = "s", long = "stop-bits", parse(try_from_str = "parse_stop_bits"),
                help = "Set number of stop bits", default_value = "1")]
    stop_bits: StopBits,

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

fn main() {
    use std::fs::File;
    use std::io::{self, BufReader, BufRead};

    let opt = Opt::from_args();
    let mut serial = serial::open(&opt.tty_path).expect("path points to invalid TTY");

    // FIXME: Implement the `ttywrite` utility.
    use serial::SerialPort;
    serial.reconfigure(&|settings| {
        settings.set_baud_rate(opt.baud_rate)?;
        settings.set_stop_bits(opt.stop_bits);
        settings.set_flow_control(opt.flow_control);
        settings.set_char_size(opt.char_width);
        Ok(())
    }).expect("configure serial error");
    SerialPort::set_timeout(&mut serial, Duration::from_secs(opt.timeout))
        .expect("timeout error");
    let reader = match opt.input {
        Some(f) => {
            let mut reader = BufReader::new(File::open(f).expect("file not found"));
            read_write(reader, &mut serial, opt.raw);
        },
        None => {
            let mut reader = BufReader::new(io::stdin());
            read_write(reader, &mut serial, opt.raw);
        },
    };
}

fn progress_fn(progress: Progress) {
    println!("Progress: {:?}", progress);
}

use std::io::{self, BufReader, BufRead};
fn read_write<T: BufRead>(mut reader: T, mut serial: &mut serial::SerialPort, raw: bool) -> io::Result<u64> {
    match raw {
        true => { io::copy(&mut reader, &mut serial) },
        false => { Ok(Xmodem::transmit_with_progress(reader, &mut serial, progress_fn)? as u64) },
    }
}