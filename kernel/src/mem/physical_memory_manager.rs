use itertools::{FoldWhile, Itertools};
use crate::{datastructures::{BitMap, BitMapIndex}, mem::PhysicalAddress, multiboot::{MemoryMap, MemoryType}, sync::SpinLock};

unsafe extern "C" {
    static __KERNEL_PHYSICAL_END: u8;
}

fn kernel_physical_end() -> PhysicalAddress {
    PhysicalAddress::from((&raw const __KERNEL_PHYSICAL_END).addr())
}

const PAGE_FRAME_BYTES: usize = 4096;

#[expect(clippy::cast_possible_truncation)]
const PAGE_FRAME_COUNT: usize =
    u64::div_exact(
        usize::MAX as u64 + 1,
        PAGE_FRAME_BYTES as u64
    ).unwrap() as usize;

type BitMapWordType = u32;
const BITMAP_WORD_BITS: usize = BitMapWordType::BITS as usize;
const BITMAP_WORD_COUNT: usize = PAGE_FRAME_COUNT.div_ceil(BITMAP_WORD_BITS);

pub struct PhysicalMemoryManager {
    page_frame_map: BitMap<BITMAP_WORD_COUNT, BitMapWordType>
}

pub static PHYSICAL_MEMORY_MANAGER: SpinLock<PhysicalMemoryManager> = SpinLock::new(
    PhysicalMemoryManager {
        page_frame_map: BitMap::from_data([0; BITMAP_WORD_COUNT])
    }
);

impl PhysicalMemoryManager {
    /// This function should only be called once.
    /// Doing so a second time may cause previously allocated pages to be handed out again.
    pub fn load_memory_map(&mut self, memory_map: &MemoryMap) {
        let available_pages = memory_map.0.iter()
            .filter(|entry| entry.memory_type == MemoryType::Available)
            .flat_map(|entry| {
                let last_address = entry.address.add_offset(entry.length - 1).unwrap();
                let first_page = usize::from(entry.address).div_ceil(PAGE_FRAME_BYTES);
                let last_page = usize::from(last_address) / PAGE_FRAME_BYTES;

                (first_page..=last_page).map(PhysicalPageFrameIndex::new)
            })
            .filter(|index| index.start_address() >= kernel_physical_end())
            .map(|index| BitMapIndex::new(index.0));

        for page in available_pages {
            self.page_frame_map.set_bit(page);
        }
    }

    pub fn allocate(&mut self, page_count: usize) -> Option<PhysicalPageFrameRange> {
        if page_count == 0 { return None; }

        let free_range = self.find_free_range(page_count)?;

        for i in 0..free_range.page_count {
            let index = BitMapIndex::new(free_range.start.0 + i);
            self.page_frame_map.clear_bit(index);
        }

        Some(free_range)
    }

    fn find_free_range(&self, free_page_frame_count: usize) -> Option<PhysicalPageFrameRange> {
        self.page_frame_map
            .enumerate()
            .fold_while(None, |range: Option<PhysicalPageFrameRange>, (page_index, page_available)| {
                if !page_available {
                    return FoldWhile::Continue(None);
                }

                let new_range = match range {
                    Some(PhysicalPageFrameRange { start, page_count: size }) => PhysicalPageFrameRange {
                        start,
                        page_count: size + 1
                    },
                    None => PhysicalPageFrameRange {
                        start: PhysicalPageFrameIndex::new(page_index.index()),
                        page_count: 1
                    }
                };

                if new_range.page_count >= free_page_frame_count {
                    FoldWhile::Done(Some(new_range))
                } else {
                    FoldWhile::Continue(Some(new_range))
                }
            }).into_inner()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PhysicalPageFrameIndex(usize);

impl PhysicalPageFrameIndex {
    const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn start_address(self) -> PhysicalAddress {
        PhysicalAddress::new(self.0 * PAGE_FRAME_BYTES)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PhysicalPageFrameRange {
    pub start: PhysicalPageFrameIndex,
    pub page_count: usize
}
