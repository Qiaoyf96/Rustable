pub fn sys_exit() {
    unsafe { asm!("svc 5" :::: "volatile"); }
}

pub fn sys_wait(pid: usize) {
    unsafe { 
        asm!("mov x0, $0
            svc 2"
            :: "r"(pid)
            : "x0", "x7"
            : "volatile"
        ); 
    }
}

pub fn sys_sleep(ms: usize) -> usize {
    let result: u32;
    unsafe { 
        asm!("mov x0, $1
            svc 1
            mov $0, x0"
            : "=r"(result)
            : "r"(ms)
            : "x0", "x7"
            : "volatile"
        ); 
    }
    result as usize
}

pub fn sys_fork() -> usize {
    let pid: u32;
    unsafe { 
        asm!("svc 4
            mov $0, x0" 
            : "=r"(pid)
            ::: "volatile"
        ); 
    }
    pid as usize
}

pub fn sys_print(num: usize) -> usize {
    let result: u32;
    unsafe { 
        asm!("mov x0, $1
            svc 3
            mov $0, x0"
            : "=r"(result)
            : "r"(num)
            : "x0", "x7":"volatile"
        ); 
    }
    result as usize
}