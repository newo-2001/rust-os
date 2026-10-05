use core::{fmt::Display, mem::MaybeUninit, ops::{Index, IndexMut}};

use crate::datastructures::BufferFullError;

#[derive(Debug)]
pub struct Stack<T, const N: usize> {
    items: [MaybeUninit<T>; N],
    length: usize
}

impl<T: Copy, const N: usize> Clone for Stack<T, N> {
    fn clone(&self) -> Self {
        Self { items: self.items, length: self.length }
    }
}

impl<T, const N: usize> Stack<T, N> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            items: [const { MaybeUninit::<T>::uninit() }; N],
            length: 0
        }
    }

    pub const fn len(&self) -> usize {
        self.length
    }

    pub const fn is_full(&self) -> bool {
        self.length == N
    }

    pub const fn is_empty(&self) -> bool {
        self.length == 0 
    }

    pub fn push_back(&mut self, value: T) -> Result<(), BufferFullError> {
        if self.is_full() {
            return Err(BufferFullError(N));
        }

        self.items[self.length].write(value);
        self.length += 1;
        Ok(())
    }

    pub const fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.length -= 1;
        let entry = &mut self.items[self.length];
        let item = core::mem::replace(entry, MaybeUninit::uninit());
        Some(unsafe { item.assume_init() })
    }

    pub const fn get(&self, index: usize) -> Option<&T> {
        if index >= self.length {
            return None;
        }

        let item = &self.items[index];
        Some(unsafe { item.assume_init_ref() })
    }

    pub fn as_slice(&self) -> &[T] {
        let slice = &self.items[0..self.length];
        unsafe { slice.assume_init_ref() }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let slice = &mut self.items[0..self.length];
        unsafe { slice.assume_init_mut() }
    }

    fn iter(&self) -> core::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }
}

impl<T, const N: usize> Default for Stack<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Index<usize> for Stack<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.length);

        let item = &self.items[index];
        unsafe { item.assume_init_ref() }
    }
}

impl<T, const N: usize> IndexMut<usize> for Stack<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.length);

        let item = &mut self.items[index];
        unsafe { item.assume_init_mut() }
    }
}

impl<T, const N: usize> From<[T; N]> for Stack<T, N> {
    fn from(value: [T; N]) -> Self {
        Self {
            length: value.len(),
            items: value.map(MaybeUninit::new),
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a Stack<T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut Stack<T, N> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, const N: usize> Display for Stack<T, N>
    where T: Display
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[")?;

        match self.as_slice() {
            &[] => {},
            slice => {
                for i in &slice[0..self.length - 1] {
                    write!(f, "{i}, ")?;
                }

                write!(f, "{}", slice[self.length - 1])?;
            }
        }

        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_back_pushes_elements() {
        let mut stack = Stack::<u32, 2>::new();
        stack.push_back(0);
        stack.push_back(1);

        assert_eq!(0, stack[0]);
        assert_eq!(1, stack[1]);
        assert_eq!(2, stack.len());
    }

    #[test]
    fn push_back_fails_on_full_stack() {
        let mut stack = Stack::<u32, 1>::new();

        stack.push_back(0);
        assert_eq!(Err(BufferFullError(1)), stack.push_back(1));
        assert!(stack.is_full());
    }

    #[test]
    fn new_stack_is_empty() {
        let stack = Stack::<u32, 1>::new();
        assert!(stack.is_empty());
    }

    #[test]
    fn get_past_length_returns_none() {
        let stack = Stack::<u32, 1>::new();
        assert_eq!(None, stack.get(0));
    }

    #[test]
    fn get_within_bounds_returns_some() {
        let stack = Stack::<u32, 1>::from([0]);

        assert_eq!(Some(&0), stack.get(0));
    }

    #[test]
    fn get_out_of_bounds_returns_none() {
        let stack = Stack::<u32, 0>::new();

        assert_eq!(None, stack.get(0));
    }

    #[test]
    fn pop_after_push_identity() {
        let mut stack = Stack::<u32, 2>::new();

        stack.push_back(0);
        stack.push_back(1);
        assert_eq!(Some(1), stack.pop_back());
        assert_eq!(1, stack.len());
    }

    #[test]
    fn as_slice() {
        let mut stack = Stack::<u32, 3>::new();
        stack.push_back(0);
        stack.push_back(1);

        assert_eq!(&[0, 1], stack.as_slice());
    }

    #[test]
    fn iter() {
        let stack = Stack::<u32, 2>::from([0, 1]);
        let mut iterator = stack.iter();

        assert_eq!([Some(&0), Some(&1), None], [iterator.next(), iterator.next(), iterator.next()]);
    }
}