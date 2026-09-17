#[used]
#[unsafe(link_section = ".gdt")]
#[unsafe(no_mangle)]
static GLOBAL_DESCRIPTOR_TABLE: [GdtEntry; 3] = [
    GdtEntry(0),
    GdtEntry::new(
        0,
        0xfffff,
        CodeAccess {
            readable: true,
            conforming: false,
            privilege_level: PrivilegeLevel::Kernel,
        },
        Flags {
            size: SegmentSize::Size32,
            granularity: Granularity::Page,
        },
    ),
    GdtEntry::new(
        0,
        0xfffff,
        DataAccess {
            writable: true,
            direction: ExpansionDirection::Up,
            privilage_level: PrivilegeLevel::Kernel,
        },
        Flags {
            size: SegmentSize::Size32,
            granularity: Granularity::Page,
        },
    ),
];

#[derive(Clone, Copy)]
enum PrivilegeLevel {
    Kernel = 0,
    User = 3,
}

#[derive(Clone, Copy)]
enum ExpansionDirection {
    Up = 0,
    Down = 1,
}

const trait Access {
    fn access_byte(self) -> u8;
}

#[derive(Clone, Copy)]
struct CodeAccess {
    pub readable: bool,
    pub conforming: bool,
    pub privilege_level: PrivilegeLevel,
}

const impl Access for CodeAccess {
    fn access_byte(self) -> u8 {
        let accessed_bit = 1u8 << 0;
        let readable_bit = u8::from(self.readable) << 1;
        let conforming_bit = u8::from(self.conforming) << 2;
        let executable_bit = 1u8 << 3;
        let descriptor_type_bit = 1u8 << 4;
        let privilage_bits = (self.privilege_level as u8) << 5;
        let present_bit = 1u8 << 7;

        accessed_bit
            | readable_bit
            | conforming_bit
            | executable_bit
            | descriptor_type_bit
            | privilage_bits
            | present_bit
    }
}

#[derive(Clone, Copy)]
struct DataAccess {
    pub writable: bool,
    pub direction: ExpansionDirection,
    pub privilage_level: PrivilegeLevel,
}

const impl Access for DataAccess {
    fn access_byte(self) -> u8 {
        let accessed_bit = 1u8 << 0;
        let writable_bit = u8::from(self.writable) << 1;
        let direction_bit = (self.direction as u8) << 2;
        let executable_bit = 0u8 << 3;
        let descriptor_type_bit = 1u8 << 4;
        let privilage_bits = (self.privilage_level as u8) << 5;
        let present_bit = 1u8 << 7;

        accessed_bit
            | writable_bit
            | direction_bit
            | executable_bit
            | descriptor_type_bit
            | privilage_bits
            | present_bit
    }
}

#[derive(Clone, Copy)]
enum Granularity {
    Byte = 0,
    Page = 1,
}

#[derive(Clone, Copy)]
enum SegmentSize {
    Size16 = 0,
    Size32 = 1,
}

#[derive(Clone, Copy)]
struct Flags {
    size: SegmentSize,
    granularity: Granularity,
}

impl Flags {
    const fn flag_bits(self) -> u8 {
        let reserved_bit = 0u8 << 0;
        let long_mode_bit = 0u8 << 1;
        let size_bit = (self.size as u8) << 2;
        let granularity_bit = (self.granularity as u8) << 3;

        reserved_bit | long_mode_bit | size_bit | granularity_bit
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct GdtEntry(u64);

impl GdtEntry {
    pub const fn new<A: [const] Access>(base: u32, limit: u32, access: A, flags: Flags) -> Self {
        assert!(limit < (1 << 20));

        let base = u64::from(base);
        let limit = u64::from(limit);

        let base_low = base & 0xffff;
        let base_mid = (base >> 16) & 0xff;
        let base_high = base >> 24;

        let limit_low = limit & 0xffff;
        let limit_high = limit >> 16;

        let access_byte = u64::from(access.access_byte());
        let flag_bits = u64::from(flags.flag_bits());

        let value = (limit_low << 0)
            | (base_low << 16)
            | (base_mid << 32)
            | (access_byte << 40)
            | (limit_high << 48)
            | (flag_bits << 52)
            | (base_high << 56);

        Self(value)
    }
}
