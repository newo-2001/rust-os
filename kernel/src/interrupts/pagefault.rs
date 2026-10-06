use core::arch::asm;

use num_enum::TryFromPrimitive;

use crate::{interrupts::InterruptStackFrame, panic};
use kernel::mem::Address;

#[derive(Clone, Copy)]
struct PageFaultErrorCode {
    address: Address,
    reason: PageFaultReason,
    access: PageFaultAccess,
    // There are other intersting fields present here,
    // will look into them as needed
}

#[derive(Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum PageFaultReason {
    NonPresentPage = 0,
    PageLevelProtectionViolation = 1,
}

#[derive(Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum PageFaultAccess {
    Read = 0,
    Write = 1,
}

impl PageFaultErrorCode {
    fn new(error_code: u32) -> Self {
        let mut cr2: u32 = 0;

        unsafe {
            asm!(
                "mov {cr2_out:e}, cr2",
                cr2_out = out(reg) cr2,
            );
        }

        #[allow(clippy::cast_possible_truncation)]
        let flags = error_code as u8;

        Self {
            address: Address::from(cr2 & (!((1 << 12) - 1))),
            reason: PageFaultReason::try_from(flags & 1).unwrap(),
            access: PageFaultAccess::try_from((flags >> 1) & 1).unwrap(),
        }
    }
}

pub extern "x86-interrupt" fn page_fault_handler(frame: InterruptStackFrame, error_code: u32) {
    panic::record_interrupt_frame(&frame);
    let error = PageFaultErrorCode::new(error_code);

    let access = match error.access {
        PageFaultAccess::Read => "read",
        PageFaultAccess::Write => "write",
    };

    let reason = match error.reason {
        PageFaultReason::NonPresentPage => "Page not present",
        PageFaultReason::PageLevelProtectionViolation => "Insufficient permissions",
    };

    panic!(
        "Page fault occurred: {reason} during {access} at {:p}\nFaulting EIP: {:p}",
        error.address,
        Address::from(frame.instruction_pointer)
    )
}