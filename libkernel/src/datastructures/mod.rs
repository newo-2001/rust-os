mod spscqueue;
mod stack;

pub use spscqueue::SpscQueue;
pub use stack::Stack;
use thiserror::Error;


#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("Buffer was already full ({0} items)")]
pub struct BufferFullError(usize);
