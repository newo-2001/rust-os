#![no_std]
#![no_main]

use core::panic::PanicInfo;

use crate::{
    term::Terminal,
    vga::{VgaColor, VgaPos, VgaTextColor},
};

mod term;
mod vga;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut term: Terminal = Terminal {
        cursor_pos: VgaPos { x: 0, y: 0 },
        cursor_color: VgaTextColor::new(VgaColor::Gray, VgaColor::Black),
    };

    term.write_string("Hello world!\n");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
