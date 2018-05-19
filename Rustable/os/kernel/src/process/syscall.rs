pub fn sys_sleep(ms: u32) -> u32 {
    let error: u64;
    let result: u32;
    unsafe {
        asm!("mov x0, $2
              svc 1
              mov $0, x0
              mov $1, x7"
              : "=r"(result), "=r"(error)
              : "r"(ms)
              : "x0", "x7")
    }
    kprintln!("Slept for {} msec", result);

    assert_eq!(error, 0);
    result
}

pub fn sys_exec(procno: u32) -> u32 {
    let error: u64;
    let result: u32;
    kprintln!("execute proc {}.", procno);
    unsafe {
        asm!("mov x0, $2
              svc 2
              mov $0, x0
              mov $1, x7"
              : "=r"(result), "=r"(error)
              : "r"(procno)
              : "x0", "x7")
    }
    kprintln!("executed proc {}.", procno);

    assert_eq!(error, 0);
    result
}