// #[allow(dead_code)] to supress the unsused warning for the values 16-255
// #[derive(Debug, Clone, Copy)] enables copy semantic and make it printable
// #[repr(u8)] variants stored as u8 (u4 would be enough, but rust has no u4)
use core::fmt::{Arguments, Result, Write};
use spin::Mutex;
use volatile::Volatile;
use x86_64;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_at(&mut self, str: &str, row: u8, col: u8, color: Color) {
        let mut i = 0;
        for byte in str.bytes() {
            self.buffer.chars[row as usize][(col + i)as usize].write(ScreenChar {
                ascii_character: byte,
                color_code: ColorCode::new(color, Color::Brown),
            });
            i += 1;
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
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
            self.buffer.chars[row][col].write(blank);
        }
    }

    #[allow(dead_code)]
    pub fn clear_screen(&mut self) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            for row in 0..BUFFER_HEIGHT {
                self.buffer.chars[row][col].write(blank);
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
    }

    #[allow(dead_code)]
    pub fn read_byte(&self, row: usize, col: usize) -> u8 {
        self.buffer.chars[row][col].read().ascii_character
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Cyan, Color::Brown),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::print(format_args!($($arg)*)));
}

macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

pub fn print(args: Arguments) {
    use core::fmt::Write;
    unsafe {
        x86_64::instructions::interrupts::disable();
        let locked = WRITER.try_lock();
        if locked.is_some() {
            let mut unwrapped = locked.expect("vga_buffer write_fmt failed");
            let _res = unwrapped.write_fmt(args);
        }
    }
}

#[allow(dead_code)]
pub fn write(str: &str) {
    use core::fmt::Write;
    WRITER
        .lock()
        .write_str(str)
        .expect("vga_buffer write failed");
}

pub fn write_at(str: &str, row: u8, col: u8, color: Color) {
    unsafe {
        x86_64::instructions::interrupts::disable();
        let locked = WRITER.try_lock();
        if locked.is_some() {
            let mut unwrapped = locked.expect("vga_buffer write_at failed");
            unwrapped.write_at(str, row, col, color);
        }
    }
    unsafe {
        x86_64::instructions::interrupts::enable();
    }
}

#[allow(dead_code)]
pub fn clear_screen() {
    WRITER.lock().clear_screen();
}

#[allow(dead_code)]
pub fn read_at(row: usize, col: usize) -> u8 {
    WRITER.lock().read_byte(row, col)
}
