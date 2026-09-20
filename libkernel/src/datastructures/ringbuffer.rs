use core::{mem::MaybeUninit, ops::Index};

use thiserror::Error;

pub struct RingBuffer<T, const N: usize> {
    buffer: [MaybeUninit<T>; N],
    length: usize,
    head: usize,
}

#[derive(Debug, Clone, Copy, Error, PartialEq, Eq)]
#[error("Buffer was already full ({0} items)")]
pub struct BufferFullError(usize);

impl<T, const N: usize> RingBuffer<T, N> {
    pub const fn new() -> Self {
        Self {
            buffer: [const { MaybeUninit::<T>::uninit() }; N],
            length: 0,
            head: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.length
    }

    pub fn is_full(&self) -> bool {
        self.length == N
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn push_back(&mut self, value: T) -> Result<(), BufferFullError> {
        if self.is_full() {
            return Err(BufferFullError(N));
        }

        let buffer_idx = self.buffer_idx(self.length);
        self.buffer[buffer_idx].write(value);
        self.length += 1;

        Ok(())
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let entry = core::mem::replace(&mut self.buffer[self.head], MaybeUninit::<T>::uninit());
        self.head = (self.head + 1) % N;
        self.length -= 1;

        Some(unsafe { entry.assume_init() })
    }

    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    const fn buffer_idx(&self, index: usize) -> usize {
        (self.head + index) % N
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.length {
            return None;
        }

        let buffer_idx = self.buffer_idx(index);
        let entry = &self.buffer[buffer_idx];
        Some(unsafe { entry.assume_init_ref() })
    }

    pub fn iter<'a>(&'a self) -> RingBufferIterator<'a, T, N> {
        self.into_iter()
    }
}

impl<T, const N: usize> Index<usize> for RingBuffer<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.length);

        let buffer_index = self.buffer_idx(index);
        let entry = &self.buffer[buffer_index];
        unsafe { entry.assume_init_ref() }
    }
}

impl<T, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        for i in 0..self.length {
            let buffer_idx = self.buffer_idx(i);
            unsafe {
                self.buffer[buffer_idx].assume_init_drop();
            }
        }
    }
}

pub struct RingBufferIterator<'a, T, const N: usize> {
    buffer: &'a RingBuffer<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for RingBufferIterator<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.buffer.get(self.index - 1)
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a RingBuffer<T, N> {
    type Item = &'a T;
    type IntoIter = RingBufferIterator<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        RingBufferIterator {
            buffer: self,
            index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    fn push_back_pushes_entry() {
        let mut buffer = RingBuffer::<u32, 2>::new();
        assert_eq!(Ok(()), buffer.push_back(1));
        assert_eq!(1, buffer[0]);
        assert_eq!(1, buffer.len());
    }

    #[test]
    fn push_back_on_full_buffer_returns_error() {
        let mut buffer = RingBuffer::<u32, 1>::new();

        buffer.push_back(1).unwrap();
        assert_eq!(Err(BufferFullError(1)), buffer.push_back(2));
    }

    #[test]
    fn pop_front_pops_oldest_element() {
        let mut buffer = RingBuffer::<u32, 2>::new();

        buffer.push_back(1).unwrap();
        buffer.push_back(2).unwrap();
        assert_eq!(Some(1), buffer.pop_front());
        assert_eq!(2, buffer[0]);
        assert_eq!(1, buffer.len());
    }

    #[test]
    fn pop_front_on_empty_buffer_returns_none() {
        let mut buffer = RingBuffer::<u32, 0>::new();
        assert_eq!(None, buffer.pop_front());
    }

    #[test]
    fn get_invalid_index_returns_none() {
        let buffer = RingBuffer::<u32, 1>::new();
        assert_eq!(None, buffer.get(0));
        assert_eq!(None, buffer.get(1));
    }

    #[test]
    fn buffer_wraps_around_after_elements_are_popped() {
        let mut buffer = RingBuffer::<u32, 3>::new();

        buffer.push_back(1).unwrap();
        buffer.push_back(2).unwrap();
        assert_eq!(Some(1), buffer.pop_front());
        assert_eq!(2, buffer[0]);

        buffer.push_back(3).unwrap();
        buffer.push_back(4).unwrap();
        assert_eq!(3, buffer[1]);
        assert_eq!(4, buffer[2]);
    }

    #[test]
    fn iter_iterates_all_populated_entries() {
        let mut buffer = RingBuffer::<u32, 2>::new();

        buffer.push_back(1).unwrap();
        buffer.push_back(2).unwrap();
        buffer.pop_front().unwrap();
        buffer.push_back(3).unwrap();

        let mut iterator = buffer.iter();
        assert_eq!(Some(&2), iterator.next());
        assert_eq!(Some(&3), iterator.next());
        assert_eq!(None, iterator.next());
    }

    #[test]
    fn iter_does_not_exceed_len() {
        let mut buffer = RingBuffer::<u32, 1>::new();
        buffer.push_back(1).unwrap();

        let mut iterator = buffer.iter();
        assert_eq!(Some(&1), iterator.next());
        assert_eq!(None, iterator.next());
    }

    #[test]
    fn clear_removes_all_elements() {
        let mut buffer = RingBuffer::<u32, 2>::new();
        buffer.push_back(1).unwrap();
        buffer.push_back(2).unwrap();
        buffer.clear();

        assert_eq!(0, buffer.length);
        assert_eq!(None, buffer.pop_front());
    }
}
