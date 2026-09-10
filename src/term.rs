use crate::vga::{self, VgaChar, VgaPos, VgaTextColor};

pub struct Terminal {
    pub cursor_pos: VgaPos,
    pub cursor_color: VgaTextColor,
}

impl Terminal {
    pub fn write_string(&mut self, str: &str) {
        for c in str.bytes() {
            self.write_char(c)
        }
    }

    pub fn write_char(&mut self, char: u8) {
        const MAX_X: u8 = vga::WIDTH as u8 - 1;
        const MAX_Y: u8 = vga::HEIGHT as u8 - 1;

        if char == b'\n' {
            self.cursor_pos = match self.cursor_pos {
                VgaPos { x: _, y: MAX_Y } => VgaPos { x: 0, y: 0 },
                VgaPos { x: _, y } => VgaPos { x: 0, y: y + 1 },
            };

            return;
        }

        let vga_char = VgaChar::new(char, self.cursor_color);
        vga::put_char(vga_char, self.cursor_pos);

        self.cursor_pos = match self.cursor_pos {
            VgaPos { x: MAX_X, y: MAX_Y } => VgaPos { x: 0, y: 0 },
            VgaPos { x: MAX_X, y } => VgaPos { x: 0, y: y + 1 },
            VgaPos { x, y } => VgaPos { x: x + 1, y },
        }
    }
}
