.section .multiboot
.align 8

multiboot_header:
    .long 0xE85250D6
    .long 0
    .long multiboot_header_end - multiboot_header
    .long -(0xE85250D6 + 0 + (multiboot_header_end - multiboot_header))

    # End tag
    .short 0
    .short 0
    .long 8

multiboot_header_end:

.section .bss
.align 16

stack:
    .skip 16384

stack_top:


.section .text
.global _start
.type _start, @function

_start:
    cli

    mov $stack_top, %esp

    # Call Rust.
    call kernel_main

1:
    hlt
    jmp 1b