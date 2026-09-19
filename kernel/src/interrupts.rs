use crate::{
    devices::pic::{END_OF_INTERRUPT, MASTER_COMMAND_PORT},
    devices::vga::{
        VGA_BUFFER, VgaChar,
        VgaColor::{Black, LightGreen},
        VgaPos, VgaTextColor,
    },
};

pub type InterruptHandler = extern "x86-interrupt" fn(&InterruptStackFrame);

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_frame: &InterruptStackFrame) {
    unsafe {
        // asm!("cli");

        let pos = VgaPos { x: 0, y: 0 };
        let char = VgaChar {
            char: b'K',
            color: VgaTextColor::new(LightGreen, Black),
        };

        VGA_BUFFER.write(char, pos);

        MASTER_COMMAND_PORT.out_byte(END_OF_INTERRUPT);
    }
}

#[repr(C)]
pub struct InterruptStackFrame {
    _private: (),
}
