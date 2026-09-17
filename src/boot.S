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


.section .rodata
.align 4

gdt_pointer:
    .word GDT_LIMIT
    .long GDT_START


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

.global load_gdt
.type load_gdt, @function

load_gdt:
    lgdt gdt_pointer
    ljmp $0x08, $reload_cs

reload_cs:
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    mov %ax, %ss

    ret

.global read_gdt
.type read_gdt, @function

read_gdt:
    sgdt gdt_readback

    movl gdt_readback + 2, %eax
    movzwl gdt_readback, %edx

    movl 4(%esp), %ecx
    movl %eax, (%ecx)

    movl 8(%esp), %ecx
    movl %edx, (%ecx)

    ret

.section .bss
.align 2
gdt_readback:
    .skip 6