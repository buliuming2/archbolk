ENTRY(_start)

SECTIONS {
    . = 1M;

    .multiboot2_header : ALIGN(8) {
        KEEP(*(.multiboot2_header))
    }

    .text.boot : ALIGN(4K) {
        KEEP(*(.text.boot))
    }

    .text.boot64 : ALIGN(4K) {
        KEEP(*(.text.boot64))
    }

    .text : ALIGN(4K) {
        *(.text .text.*)
    }

    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K) {
        *(.data .data.*)
    }

    .bss : ALIGN(4K) {
        *(COMMON)
        *(.bss .bss.*)
    }

    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
        *(.note.gnu.build-id)
    }
}
