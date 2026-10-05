use core::fmt::{Display, Pointer};

macro_rules! address_type {
    ($type:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[repr(transparent)]
        pub struct $type(usize);

        impl $type {
            pub const fn new(address: usize) -> $type {
                Self(address)
            }

            pub fn add_offset(self, offset: usize) -> Option<$type> {
                Some(Self::from(usize::from(self.0).checked_add(offset)?))
            }

            pub unsafe fn add_offset_unchecked(self, offset: usize) -> $type {
                let base = usize::from(self.0);
                Self::from(unsafe { base.unchecked_add(offset) })
            }
        }

        impl From<u32> for $type {
            fn from(value: u32) -> Self {
                Self(value as usize)
            }
        }

        impl From<$type> for u32 {
            #[allow(clippy::cast_possible_truncation)]
            fn from(value: $type) -> Self {
                value.0 as Self
            }
        }

        impl From<usize> for $type {
            fn from(value: usize) -> Self {
                Self(value)
            }
        }

        impl From<$type> for usize {
            fn from(value: $type) -> Self {
                value.0
            }
        }

        impl Pointer for $type {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:#010x}", self.0)
            }
        }

        impl Display for $type {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                Pointer::fmt(self, f)
            }
        }
    }
}

address_type!(Address);
address_type!(PhysicalAddress);