.section .text.init

.global _start

_start:
    nop
    nop
    nop
    bl      kmain
    b       halt

halt:
    wfe
    b       halt