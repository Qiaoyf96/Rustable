#define EL0 0b00
#define EL1 0b01
#define EL2 0b10
#define EL3 0b11

.section .text.init

.global _start
_start:
    // read cpu affinity, start core 0, halt rest
    mrs     x1, MPIDR_EL1
    and     x1, x1, #3
    cbz     x1, setup

halt:
    // core affinity != 0, halt it
    wfe
    b       halt

setup:
    // store the desired EL1 stack pointer in x1
    adr     x1, _start

    // read the current exception level into x0 (ref: C5.2.1)
    mrs     x0, CurrentEL
    and     x0, x0, #0b1100
    lsr     x0, x0, #2

set_stack:
    // set the current stack pointer
    mov     sp, x1

zero_bss:
    // load the start address and number of bytes in BSS section
    ldr     x1, =__bss_start
    ldr     x2, =__bss_length

zero_bss_loop:
    // zero out the BSS section, 64-bits at a time
    cbz     x2, go_kmain
    str     xzr, [x1], #8
    sub     x2, x2, #8
    cbnz    x2, zero_bss_loop

go_kmain:
    // set up exception handlers
    ldr     x2, =_vectors
    msr     VBAR_EL1, x2
    
    // jump to kmain, which shouldn't return. halt if it does
    bl      kmain
    b       halt

context_save:
    // FIXME: Save the remaining context to the stack.
    mrs     x1, TTBR0_EL1
    str     x1, [SP, #-8]!

    stp     x29, lr, [SP, #-16]!
    stp     x27, x28, [SP, #-16]!
    stp     x25, x26, [SP, #-16]!
    stp     x23, x24, [SP, #-16]!
    stp     x21, x22, [SP, #-16]!
    stp     x19, x20, [SP, #-16]!
    stp     x17, x18, [SP, #-16]!
    stp     x15, x16, [SP, #-16]!
    stp     x13, x14, [SP, #-16]!
    stp     x11, x12, [SP, #-16]!
    stp     x9, x10, [SP, #-16]!
    stp     x7, x8, [SP, #-16]!
    stp     x5, x6, [SP, #-16]!
    stp     x3, x4, [SP, #-16]!
    stp     x1, x2, [SP, #-16]!

    stp     q30, q31, [SP, #-32]!
    stp     q28, q29, [SP, #-32]!
    stp     q26, q27, [SP, #-32]!
    stp     q24, q25, [SP, #-32]!
    stp     q22, q23, [SP, #-32]!
    stp     q20, q21, [SP, #-32]!
    stp     q18, q19, [SP, #-32]!
    stp     q16, q17, [SP, #-32]!
    stp     q14, q15, [SP, #-32]!
    stp     q12, q13, [SP, #-32]!
    stp     q10, q11, [SP, #-32]!
    stp     q8, q9, [SP, #-32]!
    stp     q6, q7, [SP, #-32]!
    stp     q4, q5, [SP, #-32]!
    stp     q2, q3, [SP, #-32]!
    stp     q0, q1, [SP, #-32]!

    mrs     x2, TPIDR_EL0
    mrs     x1, SP_EL0
    stp     x1, x2, [SP, #-16]!
    mrs     x2, SPSR_EL1
    mrs     x1, ELR_EL1
    stp     x1, x2, [SP, #-16]!
    
    mrs     x1, ESR_EL1
    mov     x2, sp
    
    mov     x28, lr
    bl      handle_exception
    mov     lr, x28

.global context_restore
context_restore:
    // FIXME: Restore the context from the stack.

    ldp     x1, x2, [SP], #16
    msr     ELR_EL1, x1
    msr     SPSR_EL1, x2
    ldp     x1, x2, [SP], #16
    msr     SP_EL0, x1
    msr     TPIDR_EL0, x2
    
    ldp     q0, q1, [SP], #32
    ldp     q2, q3, [SP], #32
    ldp     q4, q5, [SP], #32
    ldp     q6, q7, [SP], #32
    ldp     q8, q9, [SP], #32
    ldp     q10, q11, [SP], #32
    ldp     q12, q13, [SP], #32
    ldp     q14, q15, [SP], #32
    ldp     q16, q17, [SP], #32
    ldp     q18, q19, [SP], #32
    ldp     q20, q21, [SP], #32
    ldp     q22, q23, [SP], #32
    ldp     q24, q25, [SP], #32
    ldp     q26, q27, [SP], #32
    ldp     q28, q29, [SP], #32
    ldp     q30, q31, [SP], #32

    ldp     x1, x2, [SP], #16
    ldp     x3, x4, [SP], #16
    ldp     x5, x6, [SP], #16
    ldp     x7, x8, [SP], #16
    ldp     x9, x10, [SP], #16
    ldp     x11, x12, [SP], #16
    ldp     x13, x14, [SP], #16
    ldp     x15, x16, [SP], #16
    ldp     x17, x18, [SP], #16
    ldp     x19, x20, [SP], #16
    ldp     x21, x22, [SP], #16
    ldp     x23, x24, [SP], #16
    ldp     x25, x26, [SP], #16
    ldp     x27, x28, [SP], #16
    ldr     x29, [SP], #8
    
    ldr     x0, [SP], #16
    msr     TTBR0_EL1, x0

    ret

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