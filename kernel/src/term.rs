use core::fmt::Write;

use crate::devices::vga::{self, VgaChar, VgaColor, VgaPos, VgaTextColor};

pub struct Terminal {
    pub cursor_pos: VgaPos,
    pub cursor_color: VgaTextColor,

    line_widths: [u8; Self::MAX_Y as usize],
}

impl Terminal {
    const MAX_X: u8 = vga::BUFFER_WIDTH as u8 - 1;
    const MAX_Y: u8 = vga::BUFFER_HEIGHT as u8 - 1;

    pub fn new() -> Self {
        Self {
            cursor_pos: VgaPos { x: 0, y: 0 },
            cursor_color: VgaTextColor::new(VgaColor::Gray, VgaColor::Black),
            line_widths: [0; Self::MAX_Y as usize],
        }
    }

    pub fn write_char(&mut self, char: u8) {
        if char == b'\n' {
            return self.newline();
        } else if char == b'\t' {
            // TODO: tab handling
            return;
        }

        self.write_raw(char);
        self.line_widths[usize::from(self.cursor_pos.y)] += 1;

        match self.cursor_pos.x {
            Self::MAX_X => self.newline(),
            x => self.cursor_pos.x = x + 1,
        };
    }

    fn newline(&mut self) {
        self.cursor_pos.x = 0;
        self.cursor_pos.y = match self.cursor_pos.y {
            Self::MAX_Y => 0,
            y => y + 1,
        };
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos.x == 0 {
            self.cursor_pos.y = self.cursor_pos.y.saturating_sub(1);
            self.cursor_pos.x = self.line_widths[usize::from(self.cursor_pos.y)];
        } else {
            self.cursor_pos.x -= 1;
        }

        let line_width = &mut self.line_widths[usize::from(self.cursor_pos.y)];
        *line_width = line_width.saturating_sub(1);

        self.write_raw(b' ');
    }

    pub fn clear(&mut self, color: VgaColor) {
        let buffer = vga::VGA_BUFFER.lock();
        buffer.fill(VgaChar {
            char: b' ',
            color: VgaTextColor::new(VgaColor::White, color),
        });
    }

    fn write_raw(&mut self, char: u8) {
        let char = VgaChar {
            char,
            color: self.cursor_color,
        };

        let buffer = vga::VGA_BUFFER.lock();
        buffer.write(char, self.cursor_pos);
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
