# 进程管理

### 第一个进程的建立

该过程与 ucore 基本相同，即通过设置 trapframe，调用 eret 完成 kernel 态到 user 态的切换，并执行用户进程：（`os/kernel/src/process/scheduler.rs`，`fn start()`）

```rust
        let mut process = Process::new().unwrap();
        process.trap_frame.sp = process.stack.top().as_u64();
        process.trap_frame.elr = shell_thread as *mut u8 as u64;
        process.trap_frame.spsr = 0b000; // To EL 0, currently only unmasking IRQ
        let tf = process.trap_frame.clone();
        
        unsafe {
            asm!("mov sp, $0
              bl context_restore
              adr lr, _start
              mov sp, lr
              mov lr, xzr
              eret" :: "r"(tf) :: "volatile");
        };
```

### 时钟中断

在 `pi/src/interrupt.rs` 中添加开时钟中断和关时钟中断的函数：（默认中断全部关闭）

```rust
    /// Enables the interrupt `int`.
    pub fn enable(&mut self, int: Interrupt) {
        let index = int as u64;
        if index < 32 {
            self.registers.enable[0].or_mask(1 << index);
        } else {
            self.registers.enable[1].or_mask(1 << (index - 32));
        }
    }

    /// Disables the interrupt `int`.
    pub fn disable(&mut self, int: Interrupt) {
        let index = int as u64;
        if index < 32 {
            self.registers.disable[0].or_mask(1 << index);
        } else {
            self.registers.disable[1].or_mask(1 << (index - 32));
        }
    }
```

在 `pi/src/timer.rs` 中实现 `tick_in()` 函数产生中断：

```rust
    pub fn tick_in(&mut self, us: u32) {
        let current_low = self.registers.CLO.read();
        let compare = current_low.wrapping_add(us);
        self.registers.COMPARE[1].write(compare); // timer 1
        self.registers.CS.or_mask(0b0010); // clear timer 1 interrupt
    }
```

由此中断处理函数便可接收时钟中断：（`os/kernel/src/traps/mod.rs`）

```rust
/// This function is called when an exception occurs. The `info` parameter
/// specifies the source and kind of exception that has occurred. The `esr` is
/// the value of the exception syndrome register. Finally, `tf` is a pointer to
/// the trap frame for the exception.
#[no_mangle]
pub extern fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    kprintln!("{:?} {:?} {}", info.source, info.kind, esr);
    if info.kind == Kind::Synchronous {
        match Syndrome::from(esr) {
            Syndrome::Brk(i) => {
                shell::shell(" [brk]$ ");
                tf.elr += 4;
            },
            Syndrome::Svc(syscall) => {
                handle_syscall(syscall, tf);
                return;
            }
            _ => {}
        }
    } else if info.kind == Kind::Irq {
        let controller = Controller::new();
        use self::Interrupt::*;
        for interrupt in [Timer1, Timer3, Usb, Gpio0, Gpio1, Gpio2, Gpio3, Uart].iter() {
            if controller.is_pending(*interrupt) {
                handle_irq(*interrupt, tf);
                return;
            }
        }
    }
    loop {
        unsafe { asm!("wfe") }
    }
}
```

可见其通过调用 `handle_irq` 完成对时钟中断的处理。



由此进程管理所需的硬件交互已经完成。接下来可以在 scheduler 里实现 round-robin 算法。

### 进程管理、进程切换

进程拥有三种状态：

- **Ready**

  A task that is ready to be executed. The scheduler will execute the task when its turn comes up.

- **Running**

  A task that is currently executing.

- **Waiting**

  A task that is waiting on an event and is not ready to be executed until that event occurs. The scheduler will check if the event has occurred when the task’s turns comes up. If the event has occurred, the task is executed. Otherwise, the task loses its turn and is checked again in the future.

在 scheduler 中，主要实现 `add()` 函数完成进程添加进进程管理队列，和 `switch()` 函数，完成就绪进程的选取：

```rust
/// Adds a process to the scheduler's queue and returns that process's ID if
    /// a new process can be scheduled. The process ID is newly allocated for
    /// the process and saved in its `trap_frame`. If no further processes can
    /// be scheduled, returns `None`.
    ///
    /// If this is the first process added, it is marked as the current process.
    /// It is the caller's responsibility to ensure that the first time `switch`
    /// is called, that process is executing on the CPU.
    fn add(&mut self, mut process: Process) -> Option<Id> {
        let id = match self.last_id {
            Some(last_id) => last_id.checked_add(1)?,
            None => 0
        };

        process.trap_frame.tpidr = id;
        self.processes.push_back(process);

        if let None = self.current {
            self.current = Some(id);
        }

        self.last_id = Some(id);
        self.last_id
    }

    /// Sets the current process's state to `new_state`, finds the next process
    /// to switch to, and performs the context switch on `tf` by saving `tf`
    /// into the current process and restoring the next process's trap frame
    /// into `tf`. If there is no current process, returns `None`. Otherwise,
    /// returns `Some` of the process ID that was context switched into `tf`.
    ///
    /// This method blocks until there is a process to switch to, conserving
    /// energy as much as possible in the interim.
    fn switch(&mut self, new_state: State, tf: &mut TrapFrame) -> Option<Id> {
        let mut current = self.processes.pop_front()?;
        let current_id = current.get_id();
        current.trap_frame = Box::new(*tf);
        current.state = new_state;
        self.processes.push_back(current);

        loop {
            let mut process = self.processes.pop_front()?;
            if process.is_ready() {
                self.current = Some(process.get_id() as Id);
                *tf = *process.trap_frame;
                process.state = State::Running;

                // Push process back into queue.
                self.processes.push_front(process);
                break;
            } else if process.get_id() == current_id {
                // We cycled the list, wait for an interrupt.
                aarch64::wfi();
            }

            self.processes.push_back(process);
        }

        self.current
    }
```

实现 `handle_irq()` 函数完成时钟中断时进程的轮换：

```rust
pub fn handle_irq(interrupt: Interrupt, tf: &mut TrapFrame) {
    match interrupt {
        Interrupt::Timer1 => {
            tick_in(TICK);
            SCHEDULER.switch(State::Ready, tf).unwrap();
        }
        _ => unimplemented!("handle_irq()"),
    }
}

```

最后，我们可以在 scheduler 的 `start()` 函数中建立进程队列，并打开时钟中断，调度算法便可以执行起来了：

```rust
/// Initializes the scheduler and starts executing processes in user space
    /// using timer interrupt based preemptive scheduling. This method should
    /// not return under normal conditions.
    pub fn start(&self) {
        *self.0.lock() = Some(Scheduler::new());
        let mut process = Process::new().unwrap();
        process.trap_frame.sp = process.stack.top().as_u64();
        process.trap_frame.elr = shell_thread as *mut u8 as u64;
        process.trap_frame.spsr = 0b000; // To EL 0, currently only unmasking IRQ
        let tf = process.trap_frame.clone();
        self.add(process);

        let mut process2 = Process::new().unwrap();
        process2.trap_frame.sp = process2.stack.top().as_u64();
        process2.trap_frame.elr = shell_thread_2 as *mut u8 as u64;
        // process2.trap_frame.spsr = 0b1101_00_0000; // To EL 0, currently only unmasking IRQ
        self.add(process2);

        Controller::new().enable(Interrupt::Timer1);
        tick_in(TICK);
        
        unsafe {
            asm!("mov sp, $0
              bl context_restore
              adr lr, _start
              mov sp, lr
              mov lr, xzr
              eret" :: "r"(tf) :: "volatile");
        };
    }
```





## 遇到的问题

- 在 `scheduler.rs` 的 `start()` 函数里，向 scheduler 的列表添加 process 时，需将 `process.trap_frame` clone 一份，将 clone 的地址作为第一个进程创建时 `context_restore` 实际读到的地址。否则因为生命周期的问题会导致 scheduler 的 mutex 一直无法释放，在下一次切换进程的时候进入死锁。