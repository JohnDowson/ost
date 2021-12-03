use alloc::format;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new(
        ColorCode::new(Color::Cyan, Color::Black),
        ColorCode::new(Color::Red, Color::Blue)
    ));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts::without_interrupts;
    without_interrupts(|| WRITER.lock().write_fmt(args).unwrap());
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl Default for ScreenChar {
    fn default() -> Self {
        Self {
            ascii_character: b' ',
            color_code: ColorCode::new(Color::Black, Color::Black),
        }
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    bar_color: ColorCode,
    buffer: &'static mut Buffer,
    back_buffer: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Writer {
    pub fn new(color_code: ColorCode, bar_color: ColorCode) -> Self {
        Self {
            column_position: 0,
            row_position: 1,
            color_code,
            bar_color,
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
            back_buffer: [[Default::default(); BUFFER_WIDTH]; BUFFER_HEIGHT],
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x12 => self.cls(),
                0x08 => self.rub_one_out(),
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
        self.update_cursor();
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(true),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line(false);
                }

                let row = self.row_position;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self, print_prompt: bool) {
        if self.row_position == BUFFER_HEIGHT - 1 {
            for row in 0..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.back_buffer[row][col] = character;
                }
            }
            for row in 1..BUFFER_HEIGHT - 1 {
                for col in 0..BUFFER_WIDTH {
                    let character = self.back_buffer[row + 1][col];
                    self.buffer.chars[row][col].write(character);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        } else {
            self.row_position += 1;
        }
        self.column_position = 0;
        if print_prompt {
            self.write_byte(b'>')
        }
    }

    fn blank(&self) -> ScreenChar {
        ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        }
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(self.blank());
        }
    }

    fn cls(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
        self.row_position = 1;
    }

    fn rub_one_out(&mut self) {
        if self.column_position >= 1 {
            let row = self.row_position;
            let col = self.column_position - 1;
            self.buffer.chars[row][col].write(self.blank());
            self.column_position -= 1;
            self.update_cursor();
        }
    }

    pub fn write_str_at(&mut self, mut col: usize, mut row: usize, s: &str) {
        for byte in s.bytes() {
            match byte {
                b'\n' => {
                    col = 0;
                    row += 1
                }
                byte => {
                    if col >= BUFFER_WIDTH {
                        col = 0;
                        row += 1
                    }

                    let color_code = self.color_code;
                    self.buffer.chars[row][col].write(ScreenChar {
                        ascii_character: byte,
                        color_code,
                    });
                    col += 1;
                }
            }
        }
    }

    pub fn write_time(&mut self, time: (u8, u8, u8)) {
        let s = format!("{:02}:{:02}:{:02}", time.0, time.1, time.2);
        let mut col = 1;
        for byte in s.bytes() {
            let color_code = self.bar_color;
            self.buffer.chars[0][col].write(ScreenChar {
                ascii_character: byte,
                color_code,
            });
            col += 1;
        }
    }

    fn update_cursor(&self) {
        use x86_64::instructions::port::Port;
        let mut port_3d4 = Port::new(0x3D4);
        let mut port_3d5 = Port::new(0x3D5);
        let pos = self.row_position * BUFFER_WIDTH + self.column_position;
        unsafe {
            port_3d4.write(0x0Fu8);
            port_3d5.write(pos as u8 & 0xFF);
            port_3d4.write(0x0Eu8);
            port_3d5.write(((pos >> 8) & 0xFF) as u8);
        };
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts::without_interrupts;

    let s = "Some test string that fits on a single line";
    without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
