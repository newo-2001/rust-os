use core::{arch::asm, marker::PhantomData};

unsafe fn in_byte(port: u16) -> u8 {
    let mut value = 0;

    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") value
        );
    }

    value
}

unsafe fn out_byte(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value
        );
    }
}

#[derive(Clone, Copy)]
pub struct IoPort<M = ModeReadWrite> {
    port: u16,
    _mode: PhantomData<M>,
}

impl<M> IoPort<M>
where
    M: Mode,
{
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            _mode: PhantomData,
        }
    }
}

impl<Mode> IoPort<Mode>
where
    Mode: Read,
{
    pub unsafe fn in_byte(self) -> u8 {
        unsafe { in_byte(self.port) }
    }
}

impl<Mode> IoPort<Mode>
where
    Mode: Write,
{
    pub unsafe fn out_byte(self, value: u8) {
        unsafe {
            out_byte(self.port, value);
        }
    }
}

pub trait Mode {}
pub trait Read {}
pub trait Write {}

#[derive(Clone, Copy)]
pub struct ModeRead;
impl Read for ModeRead {}
impl Mode for ModeRead {}

#[derive(Clone, Copy)]
pub struct ModeWrite;
impl Write for ModeWrite {}
impl Mode for ModeWrite {}

#[derive(Clone, Copy)]
pub struct ModeReadWrite;
impl Read for ModeReadWrite {}
impl Write for ModeReadWrite {}
impl Mode for ModeReadWrite {}
