#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Multiboot2 header — must be in the first 8192 bytes of the kernel image.
#[unsafe(link_section = ".multiboot2_header")]
#[used]
static MULTIBOOT2_HEADER: [u8; 24] = {
    let magic: u32 = 0xe85250d6;
    let arch: u32 = 0;
    let len: u32 = 24;
    let checksum: u32 = 0u32.wrapping_sub(magic).wrapping_sub(arch).wrapping_sub(len);

    [
        (magic >> 0) as u8, (magic >> 8) as u8, (magic >> 16) as u8, (magic >> 24) as u8,
        (arch >> 0) as u8, (arch >> 8) as u8, (arch >> 16) as u8, (arch >> 24) as u8,
        (len >> 0) as u8, (len >> 8) as u8, (len >> 16) as u8, (len >> 24) as u8,
        (checksum >> 0) as u8, (checksum >> 8) as u8, (checksum >> 16) as u8, (checksum >> 24) as u8,
        0, 0, 0, 0,
        8, 0, 0, 0,
    ]
};

/// Rust entry point — called from the 64-bit boot assembly.
#[unsafe(no_mangle)]
pub extern "C" fn _start_rust() -> ! {
    loop {}
}
