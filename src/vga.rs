use bilge::prelude::*;

pub const WIDTH: usize = 80;
pub const HEIGHT: usize = 25;

static mut VGA_BUFFER: *mut [[VgaChar; WIDTH]; HEIGHT] = 0xb8000 as _;

#[bitsize(16, hide_value, new=pub)]
#[derive(FromBits, Clone, Copy)]
pub struct VgaChar {
    pub code_point: u8,
    pub color: VgaTextColor,
}

#[derive(Clone, Copy)]
pub struct VgaPos {
    pub x: u8,
    pub y: u8,
}

#[bitsize(8, hide_value, new=pub)]
#[derive(FromBits, Clone, Copy)]
pub struct VgaTextColor {
    pub fg_color: VgaColor,
    pub bg_color: VgaColor,
}

pub fn put_char(char: VgaChar, pos: VgaPos) {
    let cell = unsafe { &mut (*VGA_BUFFER)[pos.y as usize][pos.x as usize] };

    *cell = char;
}

#[bitsize(4)]
#[derive(FromBits, Clone, Copy)]
pub enum VgaColor {
    Black = 0,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    White,
    Gray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    LightMagenta,
    Yellow,
    BrightYellow,
}
