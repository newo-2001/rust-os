use core::sync::atomic::{AtomicUsize, Ordering};
use core::{arch::asm, panic::PanicInfo};
use core::fmt::{Display, Write};

use libkernel::datastructures::Stack;
use log::error;

use crate::interrupts::InterruptStackFrame;
use crate::mem::Address;
use crate::{devices::vga::{VgaColor, VgaTextColor}, term::Terminal};

unsafe extern "C" {
    static STACK_BOTTOM: u8;
    static STACK_TOP: u8;
}

static INTERRUPT_EBP: AtomicUsize = AtomicUsize::new(0);

pub fn record_interrupt_frame(frame: &InterruptStackFrame) {
    let handler_ebp = core::ptr::from_ref(frame).addr() - core::mem::size_of::<u32>() * 2;
    INTERRUPT_EBP.store(handler_ebp, Ordering::Relaxed);
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
    let interrupt_ebp = INTERRUPT_EBP.load(Ordering::Relaxed);

    let mut ebp = core::ptr::null::<u32>();
    unsafe {
        asm!(
            "mov {ebp}, ebp",
            ebp = out(reg) ebp
        );
    }

    // A null or out-of-stack ebp marks the outermost frame
    loop {
        let is_interrupt_frame = ebp.addr() == interrupt_ebp;
        let required_bytes = core::mem::size_of::<u32>() * if is_interrupt_frame { 3 } else { 2 };

        if
            // Ensure ebp points to a valid location on the stack
            ebp.addr() < stack_bottom ||
            !ebp.is_aligned() ||
            ebp.addr() > stack_top - required_bytes
        {
            break;
        }

        // The return address is stored at [ebp+4].
        // Unless we come from an interrupt, then it is at [ebp+8]
        let return_address_offset = if is_interrupt_frame { 2 } else { 1 };
        let return_address = unsafe { *ebp.add(return_address_offset) };
        if return_address == 0 {
            break;
        }

        // Normally the return address is stored, this is the instruction *after* the calling one.
        // For interrupt frames, the faulty instruction itself is stored.
        let calling_eip_offset: i32 = if is_interrupt_frame { 0 } else { -1 };
        let calling_eip = Address::from(return_address.strict_add_signed(calling_eip_offset));

        let Ok(()) = stack.push_back(calling_eip) else { break; };

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
            "Panic! at {} line {}:{}\n{}\n\n{}",
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