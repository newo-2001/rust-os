use core::fmt::{Display, Pointer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Address(usize);

impl From<u32> for Address {
    fn from(value: u32) -> Self {
        Self(value as usize)
    }
}

impl From<Address> for u32 {
    #[allow(clippy::cast_possible_truncation)]
    fn from(value: Address) -> Self {
        value.0 as Self
    }
}

impl From<usize> for Address {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<Address> for usize {
    fn from(value: Address) -> Self {
        value.0
    }
}

impl Pointer for Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#010x}", self.0)
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Pointer::fmt(self, f)
    }
}