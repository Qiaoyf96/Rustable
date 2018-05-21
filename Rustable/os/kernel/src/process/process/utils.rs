use console::kprintln;
use std;

pub fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        unsafe { *dest.offset(i as isize) = *src.offset(i as isize); }
        i += 1;
    }

    kprintln!("memcpy: dest {:x} src {:x} n {}", dest as usize, src as usize, n);
    let bits = unsafe { std::slice::from_raw_parts_mut(dest, n) };
    kprintln!("{}", String::from_utf8_lossy(&bits));
    kprintln!("---------------------------------");

    return dest;
}

pub unsafe extern fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
       unsafe{  *s.offset(i as isize) = c as u8; }
        i += 1;
    }
    return s;
}
