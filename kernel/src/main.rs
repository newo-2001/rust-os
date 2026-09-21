#![no_std]
#![no_main]
#![feature(const_trait_impl, const_convert, abi_x86_interrupt)]

use core::panic::PanicInfo;
use core::{arch::asm, fmt::Write};

use crate::devices::keyboard::{KeyAction, KeyCode, KeyEvent};
use crate::{
    devices::pic,
    devices::vga::{VgaColor, VgaTextColor},
    term::Terminal,
};

use crate::{gdt::GdtPointer, idt::IdtPointer};

mod devices;
mod gdt;
mod idt;
mod interrupts;
mod io;
mod term;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    let mut term = Terminal::new();

    writeln!(term, "Hello world!").unwrap();

    gdt::load();

    {
        let GdtPointer { base, limit } = gdt::read();
        writeln!(term, "GDT (base: {base}, limit: {limit})").unwrap();
    }

    idt::load();

    {
        let IdtPointer { base, limit } = idt::read();
        writeln!(term, "IDT (base: {base}, limit: {limit})").unwrap();
    }

    pic::initialize();

    let master_mask = unsafe { pic::MASTER_DATA_PORT.in_byte() };
    let slave_mask = unsafe { pic::SLAVE_DATA_PORT.in_byte() };
    writeln!(term, "Master PIC mask: {:#010b}", master_mask).unwrap();
    writeln!(term, "Slave PIC mask: {:#010b}", slave_mask).unwrap();

    // Enable interrupts
    unsafe {
        asm!("sti");
    }

    writeln!(term, "Hello again!").unwrap();

    term.cursor_color = VgaTextColor::new(VgaColor::LightGreen, VgaColor::Black);

    let keyboard = &devices::keyboard::KEYBOARD;

    loop {
        if let Some(event) = keyboard.poll_event() {
            handle_key_event(event, &mut term);
        }
    }
}

fn handle_key_event(event: KeyEvent, term: &mut Terminal) {
    if event.action == KeyAction::Release {
        return;
    }

    match event.key_code {
        KeyCode::KeyBackspace => term.delete_char(),
        key_code => match devices::keyboard::key_code_to_ascii(key_code) {
            Some(ascii_char) => term.write_char(ascii_char),
            None => {}
        },
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
