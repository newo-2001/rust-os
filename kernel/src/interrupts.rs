use crate::devices::{
    keyboard,
    pic::{END_OF_INTERRUPT, MASTER_COMMAND_PORT},
};

pub type RegularInterruptHandler = extern "x86-interrupt" fn(&InterruptStackFrame);
pub type ErrorCodeInterruptHandler = extern "x86-interrupt" fn(&InterruptStackFrame, u32);

pub trait InterruptHandler {
    fn address(self) -> u32;
}

impl InterruptHandler for RegularInterruptHandler {
    fn address(self) -> u32 {
        self as *const () as u32
    }
}

impl InterruptHandler for ErrorCodeInterruptHandler {
    fn address(self) -> u32 {
        self as *const () as u32
    }
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_frame: &InterruptStackFrame) {
    keyboard::KEYBOARD.on_data();

    unsafe { MASTER_COMMAND_PORT.out_byte(END_OF_INTERRUPT) }
}

pub extern "x86-interrupt" fn double_fault_handler(_frame: &InterruptStackFrame) {
    panic!("Double fault!");
}

#[repr(C)]
pub struct InterruptStackFrame {
    _private: (),
}
