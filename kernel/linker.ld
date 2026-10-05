ENTRY(_start)

__KERNEL_PHYSICAL_START = 0x00100000;
__KERNEL_VIRTUAL_START = 0xc0000000;

SECTIONS
{
    . = __KERNEL_PHYSICAL_START;

    .multiboot ALIGN(8) :
    {
        KEEP(*(.multiboot))
    }

    .boot.text ALIGN(4K) :
    {
        *(.boot.text*)
    }

    . = __KERNEL_VIRTUAL_START + .;

    .text ALIGN(4K) : AT(ADDR(.text) - __KERNEL_VIRTUAL_START)
    {
        *(.text*)
    }

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - __KERNEL_VIRTUAL_START)
    {
        *(.rodata*)
    }

    .data ALIGN(4K) : AT(ADDR(.data) - __KERNEL_VIRTUAL_START)
    {
        *(.got*)
        *(.data*)
    }

    .bss ALIGN(4K) (NOLOAD) : AT(ADDR(.bss) - __KERNEL_VIRTUAL_START)
    {
        *(COMMON)
        *(.bss*)
    }

    __KERNEL_PHYSICAL_END = LOADADDR(.bss) + SIZEOF(.bss);

    ASSERT(
        __KERNEL_PHYSICAL_END <= 4M,
        "Kernel image exceeds the allocated 4MiB bootstrapped memory"
    )
}