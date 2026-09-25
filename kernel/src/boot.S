.intel_syntax noprefix

.section .multiboot,"a"
.align 8

multiboot_header:
    .set MAGIC, 0xE85250D6
    .set ARCHITECTURE, 0
    .set HEADER_LENGTH, multiboot_header_end - multiboot_header
    .set CHECKSUM, -[MAGIC + ARCHITECTURE + HEADER_LENGTH]

    .long MAGIC
    .long ARCHITECTURE
    .long HEADER_LENGTH
    .long CHECKSUM

    # End tag
    .short 0
    .short 0
    .long 8

multiboot_header_end:

.section .bss,"aw",@nobits
.align 16

stack:
    .skip 16384

stack_top:

.section .boot.text,"ax"
.global _start
.type _start, @function

_start:
    # Disable interrupts
    cli

    # We start by configuring our page directory.
    # Until we have enabled paging we have to translate all linked symbols
    # from their (high) virtual address to their (low) physical ones.
    .set KERNEL_PHYSICAL_START, 0x00100000
    .set KERNEL_VIRTUAL_START, 0xc0000000

    .extern PAGE_DIRECTORY
    .extern KERNEL_PAGE_TABLE

    mov eax, OFFSET PAGE_DIRECTORY
    sub eax, KERNEL_VIRTUAL_START

    mov ebx, OFFSET KERNEL_PAGE_TABLE
    sub ebx, KERNEL_VIRTUAL_START

    .set PRESENT_BIT, 1 << 0
    .set READ_WRITE_BIT, 1 << 1
    .set PERMISSION_BIT, 0 << 2
    .set PAGE_SIZE_BIT, 0 << 7
    .set KERNEL_PAGE_DIR_FLAGS, PRESENT_BIT | READ_WRITE_BIT | PERMISSION_BIT | PAGE_SIZE_BIT
    or ebx, KERNEL_PAGE_DIR_FLAGS

    # We move the kernel page table into PAGE_DIRECTORY[0]
    # We need this identity map until we have jumped to high memory
    # to avoid crashing as soon as we enable paging
    mov DWORD PTR [eax], ebx

    # We duplicate the kernel page table into the entry
    # that maps physical address 0xc00000000.
    # We now have two virtual copies of the same physical kernel memory
    mov ecx, KERNEL_VIRTUAL_START >> 22
    shl ecx, 2 # Multiply by 4 (the page dir entry size)
    mov DWORD PTR [eax + ecx], ebx

    # Move the address of the page directory into cr3
    mov cr3, eax

    # Enable paging by setting the PG bit of cr0
    .set PG_BIT, 1 << 31
    mov ebx, cr0
    or ebx, PG_BIT
    mov cr0, ebx

    # With paging enabled we can now jump to our higher half copy
    lea ecx, [1f]
    add ecx, KERNEL_VIRTUAL_START
    jmp ecx
1:

    # We can now disable the lower page directory entry
    mov DWORD PTR [eax], 0

    # Initialize the stack
    mov esp, OFFSET stack_top

    # Call Rust
    mov ecx, OFFSET kernel_main
    call ecx

2:
    hlt
    jmp 2b