use core::fmt::Write;

use crate::vga::{self, VgaChar, VgaPos, VgaTextColor};

pub struct Terminal {
    pub cursor_pos: VgaPos,
    pub cursor_color: VgaTextColor,
}

impl Terminal {
    pub fn write_char(&mut self, char: u8) {
        if char == b'\n' {
            return self.newline();
        }

        let vga_char = VgaChar {
            char,
            color: self.cursor_color,
        };

        vga::VGA_BUFFER.write(vga_char, self.cursor_pos);

        const MAX_X: u8 = vga::BUFFER_WIDTH as u8 - 1;
        match self.cursor_pos.x {
            MAX_X => self.newline(),
            x => self.cursor_pos.x = x + 1,
        };
    }

    fn newline(&mut self) {
        const MAX_Y: u8 = vga::BUFFER_HEIGHT as u8 - 1;

        self.cursor_pos.x = 0;
        self.cursor_pos.y = match self.cursor_pos.y {
            MAX_Y => 0,
            y => y + 1,
        };
    }
}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            self.write_char(c);
        }

        Ok(())
    }
}
