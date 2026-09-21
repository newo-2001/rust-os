use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};

use thiserror::Error;

/// An SpscQueue is a Single Producer Single Consumer queue.
/// Violating this assumption can lead to UB.
/// It uses interior mutability to be lock-free.
///
/// `N` is the capacity of the buffer, this is 1 less than max items it can store,
/// it needs one empty slot, meaning a buffer with `N = 2` can only store 1 item.
/// This is a limitation of const generics.
pub struct SpscQueue<T, const N: usize> {
    buffer: UnsafeCell<[MaybeUninit<T>; N]>,
    read: AtomicUsize,
    write: AtomicUsize,
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("Buffer was already full ({0} items)")]
pub struct BufferFullError(usize);

impl<T, const N: usize> SpscQueue<T, N> {
    pub const fn new() -> Self {
        assert!(N >= 2);

        Self {
            buffer: UnsafeCell::new([const { MaybeUninit::<T>::uninit() }; N]),
            read: AtomicUsize::new(0),
            write: AtomicUsize::new(0),
        }
    }

    /// Pushes an element to the back of the queue, only intende for use by the "producer".
    ///
    /// Returns [`BufferFullError`] If the buffer is full.
    pub fn push_back(&self, value: T) -> Result<(), BufferFullError> {
        let write = self.write.load(Ordering::Relaxed);
        let read = self.read.load(Ordering::Acquire);
        let next_write = (write + 1) % N;
        let is_full = next_write == read;

        if is_full {
            return Err(BufferFullError(N - 1));
        }

        let entry = unsafe { &mut *(*self.buffer.get()).as_mut_ptr().add(write) };
        entry.write(value);

        self.write.store(next_write, Ordering::Release);

        Ok(())
    }

    /// Pops an element of the front of the queue, only intended for the "consumer".
    ///
    /// Returns the popped element or [`None`] if the queue is empty.
    pub fn pop_front(&self) -> Option<T> {
        let read = self.read.load(Ordering::Relaxed);
        let write = self.write.load(Ordering::Acquire);
        let is_empty = read == write;

        if is_empty {
            return None;
        }

        let slot = unsafe { &mut *(*self.buffer.get()).as_mut_ptr().add(read) };
        let entry = core::mem::replace(slot, MaybeUninit::<T>::uninit());

        let next_read = (read + 1) % N;
        self.read.store(next_read, Ordering::Release);

        Some(unsafe { entry.assume_init() })
    }

    /// Clear the queue by repeatedly popping all elements until it is empty,
    /// this is only inteded to be used by the "consumer".
    pub fn clear(&self) {
        while self.pop_front().is_some() {}
    }

    /// Peek the nth element from the front without consuming it.
    ///
    /// Returns the nth element from the front
    /// or [`None`] if the queue contains less than `index` elements.
    pub fn peek(&self, index: usize) -> Option<T>
    where
        T: Copy,
    {
        if index >= self.len() {
            return None;
        }

        let read = self.read.load(Ordering::Relaxed);
        let index = (read + index) % N;
        let entry = unsafe { &*(*self.buffer.get()).as_ptr().add(index) };
        Some(unsafe { entry.assume_init() })
    }

    /// Obtains the number of items currently in the queue.
    /// This value should be treated as a snapshot, and should not be used for critical logic,
    /// the producer can produce another element before you get to act on this data.
    pub fn len(&self) -> usize {
        let read = self.read.load(Ordering::Relaxed);
        let write = self.write.load(Ordering::Relaxed);

        if write >= read {
            write - read
        } else {
            N - read + write
        }
    }
}

impl<T, const N: usize> Drop for SpscQueue<T, N> {
    fn drop(&mut self) {
        let buffer = unsafe { &mut *self.buffer.get() };

        for i in 0..self.len() {
            let read = self.read.load(Ordering::Relaxed);
            let index = (read + i) % N;

            unsafe {
                buffer[index].assume_init_drop();
            }
        }
    }
}

unsafe impl<T: Send, const N: usize> Sync for SpscQueue<T, N> {}
unsafe impl<T: Send, const N: usize> Send for SpscQueue<T, N> {}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    fn push_back_pushes_entry() {
        let buffer = SpscQueue::<u32, 3>::new();
        assert_eq!(Ok(()), buffer.push_back(1));
        assert_eq!(1, buffer.peek(0).unwrap());
        assert_eq!(1, buffer.len());
    }

    #[test]
    fn push_back_on_full_buffer_returns_error() {
        let buffer = SpscQueue::<u32, 2>::new();

        buffer.push_back(1).unwrap();
        assert_eq!(Err(BufferFullError(1)), buffer.push_back(2));
    }

    #[test]
    fn pop_front_pops_oldest_element() {
        let buffer = SpscQueue::<u32, 3>::new();

        buffer.push_back(1).unwrap();
        buffer.push_back(2).unwrap();
        assert_eq!(Some(1), buffer.pop_front());
        assert_eq!(2, buffer.peek(0).unwrap());
        assert_eq!(1, buffer.len());
    }

    #[test]
    fn pop_front_on_empty_buffer_returns_none() {
        let buffer = SpscQueue::<u32, 2>::new();
        assert_eq!(None, buffer.pop_front());
    }

    #[test]
    fn get_invalid_index_returns_none() {
        let buffer = SpscQueue::<u32, 2>::new();
        assert_eq!(None, buffer.peek(0));
        assert_eq!(None, buffer.peek(1));
    }

    #[test]
    fn buffer_wraps_around_after_elements_are_popped() {
        let buffer = SpscQueue::<u32, 4>::new();

        buffer.push_back(1).unwrap();
        buffer.push_back(2).unwrap();
        assert_eq!(Some(1), buffer.pop_front());
        assert_eq!(2, buffer.peek(0).unwrap());

        buffer.push_back(3).unwrap();
        buffer.push_back(4).unwrap();
        assert_eq!(3, buffer.peek(1).unwrap());
        assert_eq!(4, buffer.peek(2).unwrap());
    }
}
