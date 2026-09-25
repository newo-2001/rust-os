use libkernel::sync::SpinLock;
use num_enum::TryFromPrimitive;

pub const BUFFER_WIDTH: u8 = 80;
pub const BUFFER_HEIGHT: u8 = 25;
const VGA_BUFFER_PTR: *mut u16 = 0xc00b_8000u32 as _;

pub static VGA_BUFFER: SpinLock<VgaBuffer> = SpinLock::new(VgaBuffer { _private: () });

#[derive(Clone, Copy)]
#[repr(C)]
pub struct VgaChar {
    pub char: u8,
    pub color: VgaTextColor,
}

impl VgaChar {
    const fn bits(self) -> u16 {
        u16::from(self.char) | (u16::from(self.color.0) << 8)
    }
}

const _: () = assert!(core::mem::size_of::<VgaChar>() == 2);

#[derive(Clone, Copy)]
pub struct VgaPos {
    pub x: u8,
    pub y: u8,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct VgaTextColor(u8);

impl VgaTextColor {
    pub const fn new(fg_color: VgaColor, bg_color: VgaColor) -> Self {
        Self((bg_color as u8) << 4 | (fg_color as u8))
    }

    #[expect(unused)]
    pub fn fg_color(self) -> VgaColor {
        VgaColor::try_from_primitive(self.0 & 0xf).unwrap()
    }

    #[expect(unused)]
    pub fn bg_color(self) -> VgaColor {
        VgaColor::try_from_primitive(self.0 >> 4).unwrap()
    }
}

pub struct VgaBuffer {
    _private: (),
}

impl VgaBuffer {
    #[expect(clippy::unused_self)]
    pub fn write(&self, char: VgaChar, pos: VgaPos) {
        assert!(pos.x < BUFFER_WIDTH);
        assert!(pos.y < BUFFER_HEIGHT);

        let offset = (usize::from(pos.y) * usize::from(BUFFER_WIDTH)) + usize::from(pos.x);
        unsafe {
            let cell = VGA_BUFFER_PTR.add(offset);
            cell.write_volatile(char.bits());
        }
    }

    #[expect(clippy::unused_self)]
    pub fn fill(&self, char: VgaChar) {
        const BUFFER_SIZE: usize = usize::from(BUFFER_WIDTH) * usize::from(BUFFER_HEIGHT);
        let buffer =
            unsafe { &mut *core::ptr::slice_from_raw_parts_mut(VGA_BUFFER_PTR, BUFFER_SIZE) };

        buffer.fill(char.bits());
    }
}

#[derive(Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum VgaColor {
    Black = 0,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    Gray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    LightMagenta,
    Yellow,
    White,
}
