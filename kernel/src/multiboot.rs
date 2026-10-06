use core::fmt::Display;

use crate::datastructures::Stack;
use log::warn;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;

use crate::mem::{Address, PhysicalAddress};

#[derive(Debug, Clone, Copy, Error)]
pub enum MultibootError {
    #[error("No memory map found")]
    NoMemoryMap
}

#[derive(Debug, Clone, Copy, Error)]
pub enum MemoryMapEntryError {
    #[error("Memory map entry starts at unaddressable memory: {0:#018x}")]
    Unaddressable(u64),
    #[error("Memory map entry has unrecongized memory type: {0}")]
    UnrecognizedMemoryType(u32)
}

#[derive(Debug, Clone)]
pub struct MultibootInfo {
    pub memory_map: MemoryMap
}

#[derive(Debug, Clone)]
pub struct MemoryMap(pub Stack<MemoryMapEntry, 32>);

#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    pub address: PhysicalAddress,
    pub length: usize,
    pub memory_type: MemoryType
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RawMultiBootInfo {
    total_size: u32,
    _reserved: u32,
    tags: RawTag
}

#[derive(Clone, Copy)]
#[repr(C)]
struct RawMemoryMapEntry {
    address: u64,
    length: u64,
    memory_type: u32,
    _zero: u32
}

#[derive(Clone, Copy)]
#[repr(C)]
struct RawMemoryMapTag {
    tag: RawTag,
    entry_size: u32,
    entry_version: u32,
    entries: RawMemoryMapEntry
}

#[derive(Clone, Copy)]
#[repr(C)]
struct RawTag {
    tag_type: u32,
    size: u32
}

#[derive(Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
enum TagType {
    End = 0,
    MemoryMap = 6,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive, PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryType {
    Available = 1,
    Reserved = 2,
    ApciReclaimable = 3,
    Nvs = 4,
    BadRam = 5
}

impl TryFrom<&RawMultiBootInfo> for MultibootInfo {
    type Error = MultibootError;

    fn try_from(multiboot_info: &RawMultiBootInfo) -> Result<Self, Self::Error> {
        let mut next_tag_ptr = core::ptr::from_ref(&multiboot_info.tags);
        let mut memory_map: Option<MemoryMap> = None;

        loop {
            let tag_ptr = next_tag_ptr;
            let tag = unsafe { *tag_ptr };

            // Multiboot tags are 8 byte aligned
            let tag_size = (tag.size + (8 - tag.size % 8) % 8) as usize;
            next_tag_ptr = unsafe { tag_ptr.byte_add(tag_size) };

            let tag_type = match TagType::try_from_primitive(tag.tag_type) {
                Ok(tag_type) => tag_type,
                Err(TryFromPrimitiveError { number }) => {
                    warn!("Unrecognized multiboot tag type: {number}");
                    continue;
                }
            };

            match tag_type {
                TagType::MemoryMap => {
                    let tag = unsafe { &*tag_ptr.cast::<RawMemoryMapTag>() };

                    if memory_map.is_some() {
                        warn!("Found multiple memory map tags, using latest");
                    }

                    let _ = memory_map.insert(MemoryMap::from(tag));
                },
                TagType::End => break,
            }
        }

        Ok(Self {
            memory_map: memory_map.ok_or(MultibootError::NoMemoryMap)?
        })
    }
}

impl From<&RawMemoryMapTag> for MemoryMap {
    fn from(tag: &RawMemoryMapTag) -> Self {
        let tag_end = unsafe { core::ptr::from_ref(tag).byte_add(tag.tag.size as usize) };
        let mut entry_ptr = core::ptr::from_ref(&tag.entries);
        let mut entries = Stack::<MemoryMapEntry, 32>::new();

        while entry_ptr.addr() < tag_end.addr() {
            match MemoryMapEntry::try_from(unsafe { *entry_ptr }) {
                Err(err) => warn!("Memory map entry dropped: {err}"),
                Ok(entry) => {
                    if entries.push_back(entry).is_err() {
                        warn!("Memory map buffer full, we are potentially ignoring valid memory!");
                        break;
                    }
                }
            }

            entry_ptr = unsafe { entry_ptr.byte_add(tag.entry_size as usize) };
        }

        Self(entries)
    }
}

impl TryFrom<RawMemoryMapEntry> for MemoryMapEntry {
    type Error = MemoryMapEntryError;

    fn try_from(entry: RawMemoryMapEntry) -> Result<Self, Self::Error> {
        let orignal_length = entry.length.saturating_cast::<usize>();
        let address = entry.address.checked_cast::<usize>()
            .ok_or(MemoryMapEntryError::Unaddressable(entry.address))?;

        // Clamp the section to the addressable space
        let max_address = address.saturating_add(orignal_length);
        let length = max_address - address + 1;

        if orignal_length > length {
            warn!("Memory map entry exceeds addressable space, clamping the region");
        }

        let memory_type = MemoryType::try_from_primitive(entry.memory_type)
            .map_err(|TryFromPrimitiveError { number }| MemoryMapEntryError::UnrecognizedMemoryType(number))?;

        Ok(Self {
            address: PhysicalAddress::from(address),
            length,
            memory_type
        })
    }
}

impl Display for MemoryMapEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let high = Address::from(usize::from(self.address) + self.length);
        write!(f, "{} - {}: {:?}", self.address, high, self.memory_type)
    }
}

impl Display for MemoryMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for entry in &self.0 {
            writeln!(f, "{entry}")?;
        }
        Ok(())
    }
}

impl Display for MultibootInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Memory map:\n{}", self.memory_map)
    }
}