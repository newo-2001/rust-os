use core::{arch::asm, ptr::addr_of};

use log::trace;
use num_enum::TryFromPrimitive;

use crate::interrupts::InterruptStackFrame;

const KERNEL_MEMORY_START: u32 = 0xc000_0000;

pub static mut PAGE_DIRECTORY: PageDirectory = PageDirectory::empty();

macro_rules! static_page_table {
    ($physical_base_address:expr, $access_mode:expr, $privilege_level:expr) => {
        PageTable {
            table: const {
                let mut page_table = [PageTableEntry(0); 1024];
                let mut i: usize = 0;

                while i < 1024 {
                    let page_start_physical = $physical_base_address + (i as u32) * 4096;
                    page_table[i] =
                        PageTableEntry::new(page_start_physical, $access_mode, $privilege_level);
                    i += 1;
                }

                page_table
            },
        }
    };
}

static KERNEL_PAGE_TABLE: PageTable =
    static_page_table!(0x0, AccessMode::ReadWrite, PrivilegeLevel::Supervisor);

const fn physical_address_to_page_dir_index(address: u32) -> usize {
    (address >> 22) as usize
}

#[repr(align(4096))]
pub struct PageDirectory {
    table: [PageDirectoryEntry; 1024],
}

impl PageDirectory {
    pub const fn empty() -> Self {
        Self {
            table: [PageDirectoryEntry(0); 1024],
        }
    }

    pub fn initialize(&mut self) {
        const KERNEL_MEMORY_START_PAGE: usize =
            physical_address_to_page_dir_index(KERNEL_MEMORY_START);

        let kernel_page_dir_entry = PageDirectoryEntry::new(
            core::ptr::addr_of!(KERNEL_PAGE_TABLE) as u32,
            AccessMode::ReadWrite,
            PrivilegeLevel::Supervisor,
        );

        // Identity map for kernel memory, needed so we don't page fault immediately after enable paging
        // We will remove this mapping after initialization is complete
        self.table[0] = kernel_page_dir_entry;

        // Map the kernel into the higher half as well, we will jump execution here after initialization
        // We now have 2 copies of the kernel in virtual memory
        self.table[KERNEL_MEMORY_START_PAGE] = kernel_page_dir_entry;

        // Recursively map the page directory table itself in the last slot
        self.table[1023] = PageDirectoryEntry::new(
            core::ptr::addr_of!(PAGE_DIRECTORY) as u32,
            AccessMode::ReadWrite,
            PrivilegeLevel::Supervisor,
        );
    }

    pub fn load(&self) {
        unsafe {
            let pd_address = core::ptr::from_ref(self);
            asm! {
                "lea {next}, [2f]",

                // Load address of the page directory into CR3
                "mov cr3, {page_directory}",

                // Set the PG bit in CR0 to enable 32-bit paging
                "mov {temp}, cr0",
                "or {temp}, {pg_bit}",
                "mov cr0, {temp}",

                // Jump to the higher-half copy of the kernel
                "add {next}, {higher_half_offset}",
                "jmp {next}",
                "2:",

                page_directory = in(reg) pd_address,
                pg_bit = const (1 << 31),
                higher_half_offset = const KERNEL_MEMORY_START,
                next = out(reg) _,
                temp = out(reg) _
            }
        }

        trace!("Paging is now enabled");

        // Keep the identity map until the linker gives kernel symbols higher-half addresses.
        // The current linker script still emits absolute low addresses for Rust statics.
    }
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
    unsafe fn new() -> Self {
        let mut error_code: u32 = 0;
        let mut cr2: u32 = 0;

        unsafe {
            asm!(
                "mov {cr2_out:e}, cr2",
                "pop {error_code_out:e}",
                cr2_out = out(reg) cr2,
                error_code_out = out(reg) error_code
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

pub extern "x86-interrupt" fn page_fault_handler(_frame: &InterruptStackFrame) {
    let error = unsafe { PageFaultErrorCode::new() };

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
