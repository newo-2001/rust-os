#![no_std]
#![no_main]
#![feature(const_trait_impl, const_convert)]

use core::fmt::Write;
use core::panic::PanicInfo;

use crate::{
    term::Terminal,
    vga::{VgaColor, VgaPos, VgaTextColor},
};

mod gdt;
mod term;
mod vga;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    unsafe {
        load_gdt();
    }

    let mut term: Terminal = Terminal {
        cursor_pos: VgaPos { x: 0, y: 0 },
        cursor_color: VgaTextColor::new(VgaColor::Gray, VgaColor::Black),
    };

    writeln!(term, "Hello world!").unwrap();

    let mut gdt_base = 0;
    let mut gdt_limit = 0;
    unsafe {
        read_gdt(&mut gdt_base, &mut gdt_limit);
    }

    writeln!(term, "GDT (base: {gdt_base}, limit: {gdt_limit})").unwrap();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

unsafe extern "C" {
    fn load_gdt();
    fn read_gdt(base: *mut u32, limit: *mut u32);
}
