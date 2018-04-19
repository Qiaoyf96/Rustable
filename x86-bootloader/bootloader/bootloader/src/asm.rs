
#[inline]
pub fn inb(port: u16) -> u8 {
    let mut data: u8;
    unsafe {
        // asm!("inb %1, %0" : "=a" (data) : "d" (port));
        asm!("inb $1, $0" : "={ax}"(data) : "{dx}"(port) :: "(eax),(edx)" : "volatile")
    }
    data
}

#[inline]
pub fn outb(port: u16, data: u8) {
    unsafe {
        // asm!("outb %0, %1" :: "a" (data), "d" (port));
        asm!("outb $0, $1" :: "{ax}" (data), "{dx}" (port) :: "(eax),(edx)" : "volatile");
    }
}

#[inline]
pub fn outw(port: u16, data: u16) {
    unsafe {
        asm!("outw $0, $1" :: "{ax}" (data), "{dx}" (port) :: "(eax),(edx)" : "volatile");
    }
}

#[inline]
pub fn insl(port: u32, mut addr: u32, mut cnt: i32) {
    unsafe {
        asm!("cld;");
        // asm!(
        //     "repne; insl;"
        //     : "=D" (addr), "=c" (cnt)
        //     : "d" (port), "0" (addr), "1" (cnt)
        //     : "memory", "cc");
        asm!(
            "repne; insl;"
            : "={dx}" (addr), "={cx}" (cnt)
            : "{ax}" (port), "0" (addr), "1" (cnt)
            : "memory", "cc");
    }
}

#[inline]
pub fn jump_to(addr: usize) -> ! {
    unsafe {
        asm!("call $0" : : "{ax}"(addr) :: "intel");
        loop {}
    }
}