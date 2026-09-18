#![no_std]
#![no_main]
#![feature(const_trait_impl, const_convert, abi_x86_interrupt)]

use core::panic::PanicInfo;
use core::{arch::asm, fmt::Write};

use crate::{
    gdt::GdtPointer,
    idt::IdtPointer,
    term::Terminal,
    vga::{VgaColor, VgaPos, VgaTextColor},
};

mod gdt;
mod idt;
mod interrupts;
mod io;
mod pic;
mod term;
mod vga;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    let mut term: Terminal = Terminal {
        cursor_pos: VgaPos { x: 0, y: 0 },
        cursor_color: VgaTextColor::new(VgaColor::Gray, VgaColor::Black),
    };

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

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
