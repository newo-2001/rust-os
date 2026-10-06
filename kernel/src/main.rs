#![no_std]
#![no_main]
#![feature(const_trait_impl, const_convert, abi_x86_interrupt, exact_div)]

use core::{arch::{asm, global_asm}, fmt::Write};

global_asm!(include_str!("boot.asm"));

use log::{info, trace};

use kernel::multiboot::{RawMultiBootInfo, MultibootInfo};
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
mod panic;
mod term;

#[expect(clippy::missing_panics_doc, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "cdecl" fn kernel_main(multiboot_info: *const RawMultiBootInfo) -> ! {
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

    let multiboot_info = unsafe { multiboot_info.as_ref() }.unwrap();
    let multiboot_info = MultibootInfo::try_from(multiboot_info).unwrap();

    {
        let mut pmm = kernel::mem::PHYSICAL_MEMORY_MANAGER.lock();
        pmm.load_memory_map(&multiboot_info.memory_map);
    }

    // Enable interrupts
    unsafe {
        asm!("sti");
    }
    trace!("Interrupts enabled");

    {
        let mut pmm = kernel::mem::PHYSICAL_MEMORY_MANAGER.lock();
        let my_pages = pmm.allocate(3).unwrap();
        log::debug!("{my_pages:?}");
    };

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