use core::fmt;

/// VGA text mode color.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;

/// A minimal spinlock for the VGA writer.
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> SpinLock<T> {
        SpinLock {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> &mut T {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        unsafe { &mut *self.data.get() }
    }

    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

/// A writer for the VGA text buffer.
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: *mut [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

unsafe impl Send for Writer {}

impl Writer {
    /// Write a single byte to the buffer.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                unsafe {
                    (*self.buffer)[row][col] = ScreenChar {
                        ascii_character: byte,
                        color_code: self.color_code,
                    };
                }
                self.column_position += 1;
            }
        }
    }

    /// Scroll the screen up by one row.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                unsafe {
                    (*self.buffer)[row - 1][col] = (*self.buffer)[row][col];
                }
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            unsafe {
                (*self.buffer)[row][col] = blank;
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

/// Global writer instance.
pub static WRITER: SpinLock<Writer> = SpinLock::new(Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::Green, Color::Black),
    buffer: 0xb8000 as *mut _,
});

/// Print formatted string to the VGA buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        $crate::vga_buffer::WRITER.lock().write_fmt(format_args!($($arg)*)).unwrap();
        $crate::vga_buffer::WRITER.unlock();
    }};
}

/// Print formatted string with a newline to the VGA buffer.
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => { $crate::print!("{}\n", format_args!($($arg)*)) };
}
