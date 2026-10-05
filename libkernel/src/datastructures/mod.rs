mod spscqueue;
mod stack;
mod bitmap;

pub use spscqueue::SpscQueue;
pub use stack::Stack;
pub use bitmap::{BitMap, BitMapIndex};

use thiserror::Error;

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("Buffer was already full ({0} items)")]
pub struct BufferFullError(usize);
