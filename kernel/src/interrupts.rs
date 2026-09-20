use crate::devices::{
    keyboard,
    pic::{END_OF_INTERRUPT, MASTER_COMMAND_PORT},
};

pub type InterruptHandler = extern "x86-interrupt" fn(&InterruptStackFrame);

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_frame: &InterruptStackFrame) {
    let kb = unsafe { (&raw mut keyboard::KEYBOARD).as_mut() }.unwrap();

    kb.on_data();

    unsafe { MASTER_COMMAND_PORT.out_byte(END_OF_INTERRUPT) }
}

#[repr(C)]
pub struct InterruptStackFrame {
    _private: (),
}
