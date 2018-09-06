use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

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
struct Style(u8);

impl Style {
    const fn new(foreground: Color, background: Color) -> Style {
        Style((background as u8) << 4 | (foreground as u8))
    }
    const fn blank(&self) -> Char {
        Char {
            byte: b' ',
            style: *self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct Char {
    byte: u8,
    style: Style,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

type Buffer = [[Volatile<Char>; BUFFER_WIDTH]; BUFFER_HEIGHT];

pub struct TextScreen {
    col: usize,
    style: Style,
    buffer: &'static mut Buffer,
}

impl TextScreen {
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer[row][col].read();
                self.buffer[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.col = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = self.style.blank();
        self.buffer[row].iter_mut().for_each(|c| c.write(blank));
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row)
        }
    }

    pub fn set_font_color(&mut self, font_color: Color) {
        self.style = Style::new(font_color, Color::Black);
    }
}

impl fmt::Write for TextScreen {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            let byte = match c {
                '\n' => {
                    self.new_line();
                    continue;
                }
                ' '...'~' => c as u8,
                _ => 0xfe, // Print ■ for unprintable characters
            };

            if self.col >= BUFFER_WIDTH {
                self.new_line()
            }
            self.buffer[BUFFER_HEIGHT - 1][self.col].write(Char {
                byte,
                style: self.style,
            });
            self.col += 1;
        }
        Ok(())
    }
}

const DEFAULT_STYLE: Style = Style::new(Color::Yellow, Color::Black);

lazy_static! {
    pub static ref SCREEN: Mutex<TextScreen> = Mutex::new(TextScreen {
        col: 0,
        style: DEFAULT_STYLE,
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[cfg(test)]
mod test {
    extern crate std;

    use super::*;
    use core::fmt::Write;

    fn blank_buffer() -> Buffer {
        use array_init::array_init;
        array_init(|_| array_init(|_| Volatile::new(DEFAULT_STYLE.blank())))
    }

    fn blank_screen() -> TextScreen {
        use std::boxed::Box;
        TextScreen {
            col: 0,
            style: DEFAULT_STYLE,
            buffer: Box::leak(Box::new(blank_buffer())),
        }
    }

    #[test]
    fn write_str() {
        let mut screen = blank_screen();
        write!(screen, "XY");

        for (i, row) in screen.buffer.iter().enumerate() {
            for (j, c) in row.iter().enumerate() {
                let c = c.read();
                match (i, j) {
                    (_, 0) if i == BUFFER_HEIGHT - 1 => {
                        assert_eq!(c.byte, b'X');
                    }
                    (_, 1) if i == BUFFER_HEIGHT - 1 => {
                        assert_eq!(c.byte, b'Y');
                    }
                    _ => assert_eq!(c.byte, b' '),
                };
                assert_eq!(c.style, screen.style);
            }
        }
    }

    #[test]
    fn write_lines() {
        let mut screen = blank_screen();
        writeln!(screen, "a");
        writeln!(screen, "b{}", "c");

        for (i, row) in screen.buffer.iter().enumerate() {
            for (j, c) in row.iter().enumerate() {
                let c = c.read();
                match (i, j) {
                    (_, 0) if i == BUFFER_HEIGHT - 3 => {
                        assert_eq!(c.byte, b'a');
                    }
                    (_, 0) if i == BUFFER_HEIGHT - 2 => {
                        assert_eq!(c.byte, b'b');
                    }
                    (_, 1) if i == BUFFER_HEIGHT - 2 => {
                        assert_eq!(c.byte, b'c');
                    }
                    _ => assert_eq!(c, screen.style.blank()),
                };
                assert_eq!(c.style, screen.style);
            }
        }
    }
}
