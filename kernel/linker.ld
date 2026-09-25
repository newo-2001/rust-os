ENTRY(_start)

KERNEL_PHYSICAL_START = 0x00100000;
KERNEL_VIRTUAL_START = 0xc0000000;

SECTIONS
{
    . = KERNEL_PHYSICAL_START;

    .multiboot ALIGN(8) :
    {
        KEEP(*(.multiboot))
    }

    .boot.text ALIGN(4K) :
    {
        *(.boot.text*)
    }

    . = KERNEL_VIRTUAL_START + .;

    .text ALIGN(4K) : AT(ADDR(.text) - KERNEL_VIRTUAL_START)
    {
        *(.text*)
    }

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - KERNEL_VIRTUAL_START)
    {
        *(.rodata*)
    }

    .data ALIGN(4K) : AT(ADDR(.data) - KERNEL_VIRTUAL_START)
    {
        *(.got*)
        *(.data*)
    }

    .bss ALIGN(4K) (NOLOAD) : AT(ADDR(.bss) - KERNEL_VIRTUAL_START)
    {
        *(COMMON)
        *(.bss*)
    }
}