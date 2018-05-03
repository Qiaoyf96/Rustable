# sys_sleep() 实现

有了进程调度算法，`sys_sleep()` 便可使用进程调度 + 时钟中断的方式实现。

`sys_sleep()` 可由用户调用，其实现如下：

```rust
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
```

可见，其调用了汇编指令 `svc 1`。如此，会触发中断，并进入 `handle_exception()` 函数。在该函数中，`esr` 经过解析，会被识别出 `svc 1` 指令，并调用 `handle_syscall()` 继续处理：

```rust
pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    match num {
        1 => {
            sleep(tf.x0 as u32, tf);
        },
        _ => {
            // x7 = 1, syscall does not exist.
            tf.x1to29[6] = 1;
        }
    }
}
```

若 syscall 的参数为 1，则表示这是一个 `sleep` 操作。因此，`handle_syscall()` 具体调用 `sleep()` 函数完成该功能：

```rust
pub fn sleep(ms: u32, tf: &mut TrapFrame) {
    let begin = current_time();
    let time = begin + ms as u64 * 1000;
    let polling_fn = Box::new(move |process: &mut Process| {
        let current = current_time();
        if current > time {
            process.trap_frame.x1to29[6] = 0; // x7 = 0; succeed
            process.trap_frame.x0 = (current - begin) / 1000; // x0 = elapsed time in ms
            true
        } else {
            false
        }
    });
    SCHEDULER.switch(State::Waiting(polling_fn), tf).unwrap();
}
```

可见，其本质使用函数闭包 `polling_fn`，通过调用 `current_time()` 判断 `sleep` 过程是否结束。若结束，则返回 `true`，表示 `Waiting` 状态结束，进程在 `SCHEDULER` 的 `is_ready()` 中判断为 true，表示可以变成 `Ready` 态并被调度，否则进程继续等待。

### 遇到的问题

在实现时，突然发现之前做的 `pi::timer::current_time()` 有问题：其返回的时间永远是 0。经过 debug，我发现我将：

```rust
        let lo = self.registers.CLO.read() as u64;
        let hi = self.registers.CHI.read() as u64;
        (hi << 32) + lo
```

写成了：

```rust
        let lo = self.registers.CLO.read() as u64;
        let hi = self.registers.CHI.read() as u64;
        hi << 32 + lo
```

查资料发现，`<<` 的优先级低于 `+`。（c++ 中也是如此）