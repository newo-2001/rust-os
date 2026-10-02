use core::{arch::asm, panic::PanicInfo};
use core::fmt::{Display, Write};

use libkernel::datastructures::Stack;
use log::error;

use crate::mem::Address;
use crate::{devices::vga::{VgaColor, VgaTextColor}, term::Terminal};

unsafe extern "C" {
    static STACK_BOTTOM: u8;
    static STACK_TOP: u8;
}

struct StackTrace(Stack<Address, 20>);

impl Display for StackTrace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Stack trace:")?;
        for address in &self.0 {
            writeln!(f, "  {address}")?;
        }
        
        Ok(())
    }
}

fn stack_trace() -> StackTrace {
    let mut stack = Stack::new();
    let stack_bottom = (&raw const STACK_BOTTOM).addr();
    let stack_top = (&raw const STACK_TOP).addr();

    let mut ebp = core::ptr::null::<u32>();
    unsafe {
        asm!(
            "mov {ebp}, ebp",
            ebp = out(reg) ebp
        );
    }

    // A null or out-of-stack ebp marks the outermost frame
    while (stack_bottom..stack_top).contains(&ebp.addr()) {
        // The return address is stored right above ebp
        let return_address = unsafe { *ebp.add(1) };
        if return_address == 0 {
            break;
        }

        let Ok(()) = stack.push_back(Address::from(return_address)) else { break; };

        // The caller's ebp is saved at the address ebp points to
        ebp = unsafe { *ebp } as *const u32;
    }

    StackTrace(stack)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let stack_trace = stack_trace();

    // We can't use Option::map_or_else, because format_args! borrows a temporary
    #[expect(clippy::option_if_let_else)]
    let args = if let Some(location) = info.location() {
        format_args!(
            "Panic! at {} line {}:{}\n{}\n\nStack trace: {}",
            location.file(),
            location.line(),
            location.column(),
            info.message(),
            stack_trace
        )
    } else {
        format_args!("Panic! {}\n\nStack trace: {}", info.message(), stack_trace)
    };

    error!("{args}");

    Terminal::clear(VgaColor::Blue);
    let mut term = Terminal::new();
    term.cursor_color = VgaTextColor::new(VgaColor::White, VgaColor::Blue);

    writeln!(term, "{args}").unwrap();

    loop {}
}