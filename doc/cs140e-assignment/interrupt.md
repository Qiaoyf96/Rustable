# 中断

启动时在 `os/kernel/ext/init.S` 中切换特权级：

```assembly
    adr     x2, set_stack
    msr     ELR_EL2, x2
    eret
```

设置中断向量表（16项，每项最多包含 16 条指令）：

```assembly
#define HANDLER(source, kind) \
    .align 7; \
    stp     lr, x0, [SP, #-16]!; \
    mov     x0, ##source; \
    movk    x0, ##kind, LSL #16; \
    bl      context_save; \
    ldp     lr, x0, [SP], #16; \
    eret

.align 11
_vectors:
    // FIXME: Setup the 16 exception vectors.
    HANDLER(0, 0)
    HANDLER(0, 1)
    HANDLER(0, 2)
    HANDLER(0, 3)

    HANDLER(1, 0)
    HANDLER(1, 1)
    HANDLER(1, 2)
    HANDLER(1, 3)
    
    HANDLER(2, 0)
    HANDLER(2, 1)
    HANDLER(2, 2)
    HANDLER(2, 3)
    
    HANDLER(3, 0)
    HANDLER(3, 1)
    HANDLER(3, 2)
    HANDLER(3, 3)
```

其定义如下：

The four types of exceptions are:

- **Synchronous** - an exception resulting from an instruction like `svc` or `brk`
- **IRQ** - an asynchronous interrupt request from an external source
- **FIQ** - an asynchronous *fast* interrupt request from an external source
- **SError** - a “system error” interrupt

The four sources are:

- Same exception level when source `SP = SP_EL0`
- Same exception level when source `SP = SP_ELx`
- Lower exception level running on AArch64
- Lower exception level running on AArch32

### 中断处理

发生中断时，硬件会找到中断向量表，执行宏`HANDLER`，跳到 context_save。

首先在 `context_save` 中保存所有 caller-saved 寄存器，要按照如下格式压栈（需在 	`handle_exception()` 中将这部分内容作为 trap_frame 结构体）：
![trap-frame](./trap-frame.svg)

然后在 context_save 中设置好 esr、info（上述 4 种 source 和 4 种 kind），调用 `handle_exception()` 即可。

`handle_exception()` 需对 info.kind 进行类型判断。若是软中断，则目前先新建一个 shell 用于 debug；若是时钟中断，则调用 `handle_irq()`；并完成对 esr 解析。

该函数执行完毕后，中断处理结束，需要回到 `context_restore` 中从 tf （栈上）恢复寄存器值，回到 `HANDLER` 并 `eret`。



## 遇到的问题

- 在访问非法内存时，会触发 `brk 1` 中断。

