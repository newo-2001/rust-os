use core::arch::asm;

use num_enum::TryFromPrimitive;

use crate::interrupts::InterruptStackFrame;

#[unsafe(no_mangle)]
pub static mut PAGE_DIRECTORY: PageDirectory = PageDirectory {
    table: [PageDirectoryEntry(0); 1024],
};

#[unsafe(no_mangle)]
static KERNEL_PAGE_TABLE: PageTable = PageTable {
    table: const {
        let mut page_table = [PageTableEntry(0); 1024];
        let mut i: usize = 0;

        while i < 1024 {
            let page_start_physical = (i as u32) * 4096;
            page_table[i] = PageTableEntry::new(
                page_start_physical,
                AccessMode::ReadWrite,
                PrivilegeLevel::Supervisor,
            );
            i += 1;
        }

        page_table
    },
};

#[repr(align(4096))]
pub struct PageDirectory {
    table: [PageDirectoryEntry; 1024],
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageDirectoryEntry(u32);

impl PageDirectoryEntry {
    pub const fn new(
        physical_base_address: u32,
        access_mode: AccessMode,
        privilege_level: PrivilegeLevel,
    ) -> Self {
        // Addresses should be 4Kib aligned
        assert!(physical_base_address & ((1 << 12) - 1) == 0);

        let present = true;
        let page_size = PageSize::PageTable;
        let CacheModeBits { pcd, pwt } = CacheMode::Disabled.bits();

        let value = (u32::from(present) << 0)
            | ((access_mode as u32) << 1)
            | ((privilege_level as u32) << 2)
            | (u32::from(pwt) << 3)
            | (u32::from(pcd) << 4)
            | ((page_size as u32) << 7)
            | physical_base_address;

        Self(value)
    }
}

#[repr(align(4096))]
pub struct PageTable {
    table: [PageTableEntry; 1024],
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u32);

impl PageTableEntry {
    pub const fn new(
        base_address: u32,
        access_mode: AccessMode,
        privilege_level: PrivilegeLevel,
    ) -> Self {
        assert!(base_address & ((1 << 12) - 1) == 0);

        let present = true;
        let CacheModeBits { pcd, pwt } = CacheMode::Disabled.bits();

        let value = (u32::from(present) << 0)
            | ((access_mode as u32) << 1)
            | ((privilege_level as u32) << 2)
            | (u32::from(pwt) << 3)
            | (u32::from(pcd) << 4)
            | base_address;

        Self(value)
    }
}

#[derive(Clone, Copy)]
pub enum AccessMode {
    ReadOnly = 0,
    ReadWrite = 1,
}

#[derive(Clone, Copy)]
pub enum PrivilegeLevel {
    Supervisor = 0,
    User = 1,
}

#[derive(Clone, Copy)]
enum CacheMode {
    Disabled,
    WriteThrough,
    WriteBack,
}

#[derive(Clone, Copy)]
struct CacheModeBits {
    pcd: bool,
    pwt: bool,
}

impl CacheMode {
    const fn bits(self) -> CacheModeBits {
        match self {
            CacheMode::Disabled => CacheModeBits {
                pcd: false,
                pwt: false,
            },
            CacheMode::WriteBack => CacheModeBits {
                pcd: true,
                pwt: false,
            },
            CacheMode::WriteThrough => CacheModeBits {
                pcd: true,
                pwt: true,
            },
        }
    }
}

#[derive(Clone, Copy)]
enum PageSize {
    PageTable = 0,
    Mb2 = 1,
}

#[derive(Clone, Copy)]
struct PageFaultErrorCode {
    address: u32,
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

        let flags = error_code as u8;

        Self {
            address: cr2 & (!((1 << 12) - 1)),
            reason: PageFaultReason::try_from(flags & (1 << 0)).unwrap(),
            access: PageFaultAccess::try_from(flags & (1 << 1)).unwrap(),
        }
    }
}

pub extern "x86-interrupt" fn page_fault_handler(_frame: &InterruptStackFrame, error_code: u32) {
    log::trace!("Welcome to the page fault handler!");

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
        "Page fault occurred: {reason} during {access} at {:#<018}",
        error.address
    )
}
