use crate::{
    devices::pic::{END_OF_INTERRUPT, MASTER_COMMAND_PORT},
    devices::vga::{
        VGA_BUFFER, VgaChar,
        VgaColor::{Black, LightGreen},
        VgaPos, VgaTextColor,
    },
    io::IoPort,
};

pub type InterruptHandler = extern "x86-interrupt" fn(&InterruptStackFrame);

const KEYBOARD_DATA_PORT: IoPort = IoPort::new(0x60);

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_frame: &InterruptStackFrame) {
    unsafe {
        // asm!("cli");

        let pos = VgaPos { x: 0, y: 0 };
        let char = VgaChar {
            char: b'K',
            color: VgaTextColor::new(LightGreen, Black),
        };

        VGA_BUFFER.write(char, pos);

        let _scancode = KEYBOARD_DATA_PORT.in_byte();

        MASTER_COMMAND_PORT.out_byte(END_OF_INTERRUPT);
    }
}

#[repr(C)]
pub struct InterruptStackFrame {
    _private: (),
}
