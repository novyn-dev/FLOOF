use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts::{self, without_interrupts};
use core::fmt;
use volatile::Volatile;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        row_pos: 0,
        col_pos: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    });
}

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

impl Color {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => Color::Black,
            1 => Color::Blue,
            2 => Color::Green,
            3 => Color::Cyan,
            4 => Color::Red,
            5 => Color::Magenta,
            6 => Color::Brown,
            7 => Color::LightGray,
            8 => Color::DarkGray,
            9 => Color::LightBlue,
            10 => Color::LightGreen,
            11 => Color::LightCyan,
            12 => Color::LightRed,
            13 => Color::Pink,
            14 => Color::Yellow,
            15 => Color::White,
            _ => Color::Black, // fallback
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(fg: Color, bg: Color) -> ColorCode {
        // the bg colors must be in the 4 upper bits
        // and then we have the bitwise OR to merge fg and bg into a single byte
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode,
}

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

pub struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    pub row_pos: usize,
    pub col_pos: usize,
    pub color_code: ColorCode,
    pub buffer: &'static mut Buffer
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl Writer {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            byte => {
                if self.col_pos >= BUFFER_WIDTH {
                    self.newline();
                }

                let color_code = self.color_code;

                self.buffer.chars[self.row_pos][self.col_pos].write(ScreenChar {
                    ascii_char: byte,
                    color_code,
                });
                self.col_pos += 1;
            }
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ascii byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte), // 32..=126
                // unprintable
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn newline(&mut self) {
        self.col_pos = 0;
        if self.row_pos + 1 < BUFFER_HEIGHT {
            self.row_pos += 1;
        } else {
            // scroll
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let char = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(char); // copy them to row - 1 (up)
                }
            }
            self.clear_row(BUFFER_HEIGHT-1);
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_char: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn color_fg(&mut self, color: Color) {
        let bg = Color::from_byte(self.color_code.0 >> 4); // extract bg from the high 4 bits
        self.color_code = ColorCode::new(color, bg);
    }

    pub fn color_bg(&mut self, color: Color) {
        let fg = Color::from_byte(self.color_code.0);
        self.color_code = ColorCode::new(fg, color);
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

pub fn vga_color(fg_color: Color, bg_color: Color) {
    let mut writer = WRITER.lock();
    writer.color_fg(fg_color);
    writer.color_bg(bg_color);
}

/// "TestTestTestTest" should be displayed on the screen
#[test_case]
fn println_output() {
    use core::fmt::Write;
    let s = "TestTestTestTest";

    without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");

        for (col, c) in s.chars().enumerate() {
            let screen_char = WRITER.lock().buffer.chars[writer.row_pos-1][col].read();
            assert_eq!(char::from(screen_char.ascii_char), c)
        }
    });
}
