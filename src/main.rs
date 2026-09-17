#![no_std]
#![no_main]
#![feature(const_trait_impl, const_convert)]

use core::fmt::Write;
use core::panic::PanicInfo;

use crate::{
    gdt::GdtPointer,
    idt::IdtPointer,
    term::Terminal,
    vga::{VgaColor, VgaPos, VgaTextColor},
};

mod gdt;
mod idt;
mod term;
mod vga;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    let mut term: Terminal = Terminal {
        cursor_pos: VgaPos { x: 0, y: 0 },
        cursor_color: VgaTextColor::new(VgaColor::Gray, VgaColor::Black),
    };

    writeln!(term, "Hello world!").unwrap();

    gdt::load_gdt();

    {
        let GdtPointer { base, limit } = gdt::read_gdt();
        writeln!(term, "GDT (base: {base}, limit: {limit})").unwrap();
    }

    idt::load_idt();

    {
        let IdtPointer { base, limit } = idt::read_idt();
        writeln!(term, "IDT (base: {base}, limit: {limit})").unwrap();
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
