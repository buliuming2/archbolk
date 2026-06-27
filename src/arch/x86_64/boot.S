.section .text.boot, "ax"

.code32
.globl _start
_start:
    # Multiboot2: eax = magic (0x36d76289), ebx = multiboot_info addr
    movl    %eax, %edi
    movl    %ebx, %esi

    # Set up stack
    movl    $0x7000, %esp

    # Check multiboot magic
    cmpl    $0x36d76289, %eax
    jne     .Lfail

    # Disable interrupts
    cli

    # Enable A20 gate
    inb     $0x92, %al
    orb     $2, %al
    outb    %al, $0x92

    # Load page table at 0x1000 (identity map first 2 MiB)
    movl    $0x1000, %edi
    movl    %edi, %cr3

    # Clear 4 pages (PML4 + PDP + 2 PD entries)
    xorl    %eax, %eax
    movl    $4096, %ecx
    rep     stosb

    # PML4[0] -> PDP at 0x2000
    movl    $0x1000, %edi
    movl    $0x2003, (%edi)

    # PDP[0] -> PD at 0x3000
    movl    $0x2000, %edi
    movl    $0x3003, (%edi)

    # PD[0] -> 2 MiB huge page (0x00000083)
    movl    $0x3000, %edi
    movl    $0x00000083, (%edi)

    # Enable PAE
    movl    %cr4, %eax
    orl     $0x20, %eax
    movl    %eax, %cr4

    # Build GDT at a fixed location (0x800, below stack at 0x7000)
    # This avoids linker moving .quad constants to .rodata
    movl    $0x800, %edi
    movl    $0, (%edi)              # null descriptor (bytes 0-3)
    movl    $0, 4(%edi)             # null descriptor (bytes 4-7)
    movl    $0x00000000, 8(%edi)    # code descriptor (bytes 0-3): 0x00000000
    movl    $0x00209800, 12(%edi)   # code descriptor (bytes 4-7): 0x00209800
    movl    $0x00000000, 16(%edi)   # data descriptor (bytes 0-3): 0x00000000
    movl    $0x00209200, 20(%edi)   # data descriptor (bytes 4-7): 0x00209200

    # Load GDT descriptor pointing to 0x800
    subl    $8, %esp                # make room on stack for GDTR (6 bytes)
    movw    $23, 0(%esp)            # limit = 24 - 1 = 23
    movl    $0x800, 2(%esp)         # base = 0x800
    lgdt    (%esp)
    addl    $8, %esp                # restore stack

    # Enable long mode
    movl    $0xC0000080, %ecx
    rdmsr
    orl     $0x100, %eax
    wrmsr

    # Enable paging
    movl    %cr0, %eax
    orl     $0x80000001, %eax
    movl    %eax, %cr0

    # Far jump to 64-bit code
    pushl   $0x08
    pushl   $_start_64
    lret

.Lfail:
    cli
    hlt
    jmp     .Lfail

.section .text.boot64, "ax"
.code64
.globl _start_64
_start_64:
    # Set up segment registers
    movw    $0x10, %ax
    movw    %ax, %ds
    movw    %ax, %es
    movw    %ax, %fs
    movw    %ax, %gs
    movw    %ax, %ss

    # Set up stack
    movq    $0x7000, %rsp

    # Call Rust entry
    call    _start_rust

    # Should never return
    cli
    hlt
    jmp     .
