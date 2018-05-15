#define EL0 0b00
#define EL1 0b01
#define EL2 0b10
#define EL3 0b11

.section .text.init

.global _start

_start:
    // read cpu affinity, start core 0, halt rest
    mrs     x1, mpidr_el1
    and     x1, x1, #3
    cbz     x1, 2f

1:
    // core affinity != 0, halt it
    wfe
    b       1b

2:
    // set the stack to start before our boot code
    ldr     x1, =_start
    mov     sp, x1

    // load the start address and number of bytes in BSS section
    ldr     x1, =__bss_start
    ldr     x2, =__bss_length

3:
    // zero out the BSS section, 64-bits at a time
    cbz     x2, setup
    str     xzr, [x1], #8
    sub     x2, x2, #8
    cbnz    x2, setup

setup:
    // store the desired EL1 stack pointer in x1
    adr     x1, _start

    // read the current exception level into x0 (ref: C5.2.1)
    mrs     x0, CurrentEL
    and     x0, x0, #0b1100
    lsr     x0, x0, #2

switch_to_el2:
    // switch to EL2 if we're in EL3. otherwise switch to EL1
    cmp     x0, EL3
    bne     master

    // set-up SCR_EL3 (bits 0, 4, 5, 7, 8, 10) (A53: 4.3.42)
    mov     x2, #0x5b1
    msr     SCR_EL3, x2

    // set-up SPSR and PL switch! (bits 0, 3, 6, 7, 8, 9) (ref: C5.2.20)
    mov     x2, #0x3c9
    msr     SPSR_EL3, x2
    adr     x2, switch_to_el1
    msr     ELR_EL3, x2
    eret

master:
    //ldr x0, =0x1000000
    //mov sp, x0               // set el2 sp
    //bl vm_init
    // bl switch_to_el1

switch_to_el1:
    // switch to EL1 if we're not already in EL1. otherwise continue with start
    cmp     x0, EL1
    beq     vm

    // set the stack-pointer for EL1
    msr     SP_EL1, x1
    // bl mm_init

    // enable CNTP for EL1/EL0 (ref: D7.5.2, D7.5.13)
    // NOTE: This doesn't actually enable the counter stream.
    mrs     x0, CNTHCTL_EL2
    orr     x0, x0, #0b11
    msr     CNTHCTL_EL2, x0
    msr     CNTVOFF_EL2, xzr


    // enable AArch64 in EL1 (A53: 4.3.36)
    mov     x0, #(1 << 31)      // Enable AArch64 for EL1
    orr     x0, x0, #(1 << 1)   // RES1 on A-53
    msr     HCR_EL2, x0
    mrs     x0, HCR_EL2


    // enable floating point and SVE (SIMD) (A53: 4.3.38, 4.3.34)
    msr     CPTR_EL2, xzr     // don't trap accessing SVE registers
    mrs     x0, CPACR_EL1
    orr     x0, x0, #(0b11 << 20)
    msr     CPACR_EL1, x0


    // Set SCTLR to known state (RES1: 11, 20, 22, 23, 28, 29) (A53: 4.3.30)
    mov     x2, #0x0800
    movk    x2, #0x30d0, lsl #16
    msr     SCTLR_EL1, x2



    // change execution level to EL1 (ref: C5.2.19)
    mov     x2, #0x3c5
    msr     SPSR_EL2, x2

    // FIXME: Return to EL1 at `set_stack`.
    adr     x2, vm
    msr     ELR_EL2, x2
    eret

vm:
    bl vm_init

    mrs x0, sctlr_el1
    orr x0, x0, #(1 << 12)
    msr sctlr_el1, x0        // enable instruction cache

el1_mmu_activate:
    ldr x0, =0x04cc
    msr mair_el1, x0
    isb
    
    // Translation table base address
    ldr x1, =0x01000000
    msr ttbr0_el1, x1
    msr ttbr1_el1, x1
    isb

    // Translation Control Register, EL1
    // Controls which Translation Base Register defines the base address register for 
    // a translation table walk required for stage 1 translation of a memory access from 
    // EL0 or EL1. Also controls the translation table format and holds cacheability and 
    // shareability information.
    mrs x2, tcr_el1
    ldr x3, =0x70040ffbf
    bic x2, x2, x3

    // [34:32] b010: Intermediate Physical Address Size. 40-bit, 1TByte.
    // [31]    b1: Reseverd. Res1
    // [30]    b0: TTBR1_EL1 granule size. 4KB
    // [29:28] b11: Shareability attribute for memory associated with translation table walks using TTBR1. Inner Shareable.
    // [27:26] b11: Outer cacheability attribute for memory associated with translation table walks using TTBR1. Normal memory, Outer Write-Back no Write-Allocate Cacheable.
    // [25:24] b11: Inner cacheability attribute for memory associated with translation table walks using TTBR1. Normal memory, Inner Write-Back no Write-Allocate Cacheable.
    // [21:16] b011000: bSize offset of the memory region addressed by TTBR1
    // [13:12] b11: Shareability attribute for memory associated with translation table walks using TTBR0. Inner Shareable.
    // [11:10] b11: Outer cacheability attribute for memory associated with translation table walks using TTBR0. Normal memory, Outer Write-Back no Write-Allocate Cacheable.
    // [9:8]   b11: Inner cacheability attribute for memory associated with translation table walks using TTBR0. Normal memory, Inner Write-Back no Write-Allocate Cacheable.
    // [5:0]   b011000: Size offset of the memory region addressed by TTBR0.
    ldr x3, =0x2bf183f18
    orr x2, x2, x3
    msr tcr_el1, x2
    isb

    // System Control Register, EL1
    // Write permission implies Execute Never (XN). You can use this bit to require all 
    // memory regions with write permissions are treated as XN. The WXN bit is permitted 
    // to be cached in a TLB.
    ldr x5, =kmain
    mrs x3, sctlr_el1
    ldr x4, =0x80000
    bic x3, x3, x4

    // [12] Instruction caches enabled.
    // [2] Data and unified caches enabled.
    // [0] EL1 and EL0 stage 1 MMU enabled.
    ldr x4, =0x1005
    orr x3, x3, x4
    msr sctlr_el1, x3
    isb
    br x5



go_kmain:
    // jump to kmain, which shouldn't return. halt if it does
    bl      kmain
    b       1b
