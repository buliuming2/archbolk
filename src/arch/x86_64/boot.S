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

    # Build GDT descriptor on stack (PC-relative)
    call    .Lget_eip
.Lget_eip:
    popl    %ebx                      # ebx = runtime address of .Lget_eip
    leal    (gdt64 - .Lget_eip)(%ebx), %eax  # eax = runtime address of gdt64
    movl    %eax, .Lgdt_desc + 2      # patch the pointer in the descriptor
.Lgdt_desc:
    .word   gdt64_end - gdt64 - 1    # limit
    .quad   0                         # base (patched above)
    lgdt    .Lgdt_desc - .Lget_eip(%ebx)  # load descriptor via PC-relative

    # Enable long mode
    movl    $0xC0000080, %ecx
    rdmsr
    orl     $0x100, %eax
    wrmsr

    # Enable paging
    movl    %cr0, %eax
    orl     $0x80000001, %eax
    movl    %eax, %cr0

    # Far jump to 64-bit code (PC-relative)
    pushl   $0x08
    leal    (_start_64 - .Lget_eip)(%ebx), %eax
    pushl   %eax
    lret

.Lfail:
    cli
    hlt
    jmp     .Lfail

.section .rodata, "a"
.align 8
gdt64:
    .quad 0
    .quad 0x0020980000000000
gdt64_end:

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
