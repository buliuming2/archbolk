use core::fmt;

/// A simple serial port writer (COM1).
pub struct SerialPort {
    port: u16,
}

impl SerialPort {
    /// Create a new serial port at the given I/O port.
    pub const fn new(port: u16) -> SerialPort {
        SerialPort { port }
    }

    /// Initialize the serial port (standard 16550 init).
    pub fn init(&mut self) {
        let port = self.port;
        unsafe {
            // Disable interrupts
            core::arch::asm!("out dx, al", in("dx") port + 1, in("al") 0x00u8);
            // Enable DLAB
            core::arch::asm!("out dx, al", in("dx") port + 3, in("al") 0x80u8);
            // Set baud rate (divisor = 1, 115200 baud)
            core::arch::asm!("out dx, al", in("dx") port + 0, in("al") 0x01u8);
            core::arch::asm!("out dx, al", in("dx") port + 1, in("al") 0x00u8);
            // Disable DLAB, set 8N1
            core::arch::asm!("out dx, al", in("dx") port + 3, in("al") 0x03u8);
            // Enable FIFO
            core::arch::asm!("out dx, al", in("dx") port + 2, in("al") 0xC7u8);
            // Enable IRQs, RTS/DSR
            core::arch::asm!("out dx, al", in("dx") port + 4, in("al") 0x0Bu8);
        }
    }

    fn write_byte(&mut self, byte: u8) {
        let port = self.port;
        unsafe {
            // Wait for transmitter holding register to be empty
            let mut status: u8;
            loop {
                core::arch::asm!("in al, dx", out("al") status, in("dx") port + 5);
                if status & 0x20 != 0 {
                    break;
                }
            }
            core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
        Ok(())
    }
}

/// Global serial port instance.
pub static SERIAL: super::vga_buffer::SpinLock<SerialPort> =
    super::vga_buffer::SpinLock::new(SerialPort { port: 0x3F8 });

/// Print formatted string to the serial port.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        $crate::serial::SERIAL.lock().write_fmt(format_args!($($arg)*)).unwrap();
        $crate::serial::SERIAL.unlock();
    }};
}

/// Print formatted string with a newline to the serial port.
#[macro_export]
macro_rules! serial_println {
    () => { $crate::serial_print!("\n") };
    ($($arg:tt)*) => { $crate::serial_print!("{}\n", format_args!($($arg)*)) };
}
