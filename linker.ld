ENTRY(_start)

SECTIONS
{
    . = 1M;

    .text ALIGN(4K) :
    {
        KEEP(*(.multiboot))
        *(.text*)
    }

    .rodata ALIGN(4K) :
    {
        *(.rodata*)
    }

    .gdt ALIGN(8) :
    {
        GDT_START = .;
        KEEP(*(.gdt))
        GDT_END = .;
    }

    GDT_LIMIT = GDT_END - GDT_START - 1;

    .data ALIGN(4K) :
    {
        *(.data*)
    }

    .bss ALIGN(4K) :
    {
        *(COMMON)
        *(.bss*)
    }
}