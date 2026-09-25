#![no_std]
#![no_main]
#![feature(const_trait_impl, const_convert, abi_x86_interrupt, exact_div)]

use core::panic::PanicInfo;
use core::{arch::asm, fmt::Write};

use log::{error, info, trace};

use crate::devices::keyboard::{KeyAction, KeyCode, KeyEvent};
use crate::{
    devices::pic,
    devices::vga::{VgaColor, VgaTextColor},
    term::Terminal,
};

mod devices;
mod gdt;
mod idt;
mod interrupts;
mod io;
mod logger;
mod mem;
mod term;

#[expect(clippy::missing_panics_doc)]
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    {
        let mut com1 = devices::serial::COM1.lock();
        com1.initialize();
    }

    log::set_logger(&logger::LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::max());
    info!("Logger initialized, hello world!");

    let mut term = Terminal::new();
    writeln!(term, "Hello world!").unwrap();

    gdt::load();
    idt::load();

    pic::initialize();

    let master_mask = unsafe { pic::MASTER_DATA_PORT.in_byte() };
    let slave_mask = unsafe { pic::SLAVE_DATA_PORT.in_byte() };
    writeln!(term, "Master PIC mask: {master_mask:#010b}").unwrap();
    writeln!(term, "Slave PIC mask: {slave_mask:#010b}").unwrap();

    // Enable interrupts
    unsafe {
        asm!("sti");
    }
    trace!("Interrupts enabled");

    term.cursor_color = VgaTextColor::new(VgaColor::LightGreen, VgaColor::Black);

    loop {
        if let Some(event) = devices::keyboard::KEYBOARD.poll_event() {
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
        key_code => {
            if let Some(ascii_char) = devices::keyboard::key_code_to_ascii(key_code) {
                term.write_char(ascii_char);
            }
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // We can't use Option::map_or_else, because format_args! borrows a temporary
    #[expect(clippy::option_if_let_else)]
    let args = if let Some(location) = info.location() {
        format_args!(
            "Panic! at {} line {}:{}\n{}",
            location.file(),
            location.line(),
            location.column(),
            info.message()
        )
    } else {
        format_args!("Panic! {}", info.message())
    };

    error!("{args}");

    Terminal::clear(VgaColor::Blue);
    let mut term = Terminal::new();
    term.cursor_color = VgaTextColor::new(VgaColor::White, VgaColor::Blue);

    writeln!(term, "{args}").unwrap();

    loop {}
}
