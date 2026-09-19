use core::mem::MaybeUninit;

pub struct BufferedQueue<T: Sized, const N: usize> {
    buffer: [MaybeUninit<T>; N],
    length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferFullError;

impl<T, const N: usize> BufferedQueue<T, N>
where
    T: Sized,
{
    pub const fn new() -> Self
    where
        T: Copy,
    {
        Self {
            buffer: [MaybeUninit::<T>::uninit(); N],
            length: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), BufferFullError> {
        if self.is_full() {
            Err(BufferFullError)
        } else {
            self.push_overwriting(value);
            Ok(())
        }
    }

    pub fn push_overwriting(&mut self, value: T) -> Option<T> {
        let replaced: Option<T> = if self.is_full() { self.pop() } else { None };

        self.buffer[self.length].write(value);
        self.length += 1;

        replaced
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let entry = &mut self.buffer[0];
        let value = core::mem::replace(entry, MaybeUninit::<T>::uninit());

        for i in 1..self.length {
            let value = unsafe { self.buffer[i].assume_init_read() };
            self.buffer[i - 1].write(value);
        }

        self.length -= 1;
        Some(unsafe { value.assume_init() })
    }

    pub fn as_slice(&self) -> &[T] {
        let buffer = &self.buffer[0..self.length];
        unsafe { buffer.assume_init_ref() }
    }

    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.into_iter()
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn is_full(&self) -> bool {
        self.length == N
    }
}

impl<T: Sized, const N: usize> Drop for BufferedQueue<T, N> {
    fn drop(&mut self) {
        for entry in self.buffer.iter_mut().take(self.length) {
            unsafe {
                entry.assume_init_drop();
            }
        }
    }
}

impl<'a, T: Sized, const N: usize> IntoIterator for &'a BufferedQueue<T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pushing_into_full_queue_returns_error() {
        let mut queue = BufferedQueue::<u32, 1>::new();

        assert_eq!(Ok(()), queue.push(1));
        assert_eq!(Err(BufferFullError), queue.push(1));
    }

    #[test]
    fn test_popping_from_queue_pops_oldest_entry() {
        let mut queue = BufferedQueue::<u32, 2>::new();

        assert_eq!(Ok(()), queue.push(1));
        assert_eq!(Ok(()), queue.push(2));
        assert_eq!(Some(1), queue.pop());
        assert_eq!(Some(2), queue.pop());
    }

    #[test]
    fn test_popping_empty_queue_returns_none() {
        let mut queue = BufferedQueue::<u32, 0>::new();

        assert_eq!(None, queue.pop());
    }

    #[test]
    fn test_push_overwriting_on_full_buffer_overwrites_oldest_entry() {
        let mut queue = BufferedQueue::<u32, 2>::new();
        assert_eq!(Ok(()), queue.push(1));
        assert_eq!(Ok(()), queue.push(2));
        assert_eq!(Some(1), queue.push_overwriting(3));
        assert_eq!(&[2, 3], queue.as_slice());
    }

    #[test]
    fn test_push_overwriting_on_non_full_buffer_does_not_overwrite() {
        let mut queue = BufferedQueue::<u32, 2>::new();
        assert_eq!(Ok(()), queue.push(1));
        assert_eq!(None, queue.push_overwriting(2));
        assert_eq!(&[1, 2], queue.as_slice());
    }
}
