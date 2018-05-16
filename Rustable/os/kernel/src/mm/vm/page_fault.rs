use traps::syndrome::Fault;
use console::kprintln;

pub fn do_pgfault(kind: Fault, level: u8) {
    kprintln!("pg_fault! {:?} {}", kind, level);
}