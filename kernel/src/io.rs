use core::arch::asm;

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
        )
    }
}

#[derive(Clone, Copy)]
pub struct IoPort(u16);

impl IoPort {
    pub const fn new(port: u16) -> Self {
        Self(port)
    }

    pub unsafe fn in_byte(self) -> u8 {
        unsafe { in_byte(self.0) }
    }

    pub unsafe fn out_byte(self, value: u8) {
        unsafe {
            out_byte(self.0, value);
        }
    }
}
