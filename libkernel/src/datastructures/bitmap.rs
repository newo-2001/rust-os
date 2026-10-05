use core::{marker::PhantomData, ops::{BitAndAssign, BitOrAssign}};

use num_traits::PrimInt;

pub struct BitMap<T, const WORDS: usize> {
    data: [T; WORDS]
}

#[derive(Debug, Clone, Copy)]
pub struct BitMapIndex<T> {
    word_index: usize,
    bit_index: usize,
    _word_type: PhantomData<T>
}

impl<T: PrimInt> BitMapIndex<T> {
    #[must_use]
    pub const fn new(index: usize) -> Self {
        let bits_per_word: usize = core::mem::size_of::<T>() * 8;

        Self {
            word_index: index / bits_per_word,
            bit_index: index % bits_per_word,
            _word_type: PhantomData::<T>
        }
    }

    #[must_use]
    pub const fn index(self) -> usize {
        let word_size_bits = core::mem::size_of::<T>();
        self.word_index * word_size_bits + self.bit_index
    }
}

impl<T, const N: usize> BitMap<T, N> {
    pub const fn from_data(data: [T; N]) -> Self {
        Self { data }
    }
}

#[expect(clippy::missing_panics_doc)]
impl<T: PrimInt, const N: usize> BitMap<T, N> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: [T::zero(); N]
        }
    }

    pub fn index(&self, index: BitMapIndex<T>) -> bool {
        assert!(index.word_index < N);

        (self.data[index.word_index] >> index.bit_index) & T::one() == T::one()
    }

    pub fn get(&self, index: BitMapIndex<T>) -> Option<bool> {
        if index.word_index >= N {
            return None;
        }

        Some((self.data[index.word_index] >> index.bit_index) & T::one() == T::one())
    }

    pub fn write_bit(&mut self, index: BitMapIndex<T>, value: bool) {
        assert!(index.word_index < N);

        let value = if value { T::one() } else { T::zero() };
        let current_value = self.data[index.word_index];
        let masked = current_value & !(T::one() << index.bit_index);
        let new_value = masked | (value << index.bit_index);
        
        self.data[index.word_index] = new_value;
    }

    pub fn set_bit(&mut self, index: BitMapIndex<T>) where T: BitOrAssign {
        assert!(index.word_index < N);

        self.data[index.word_index] |= T::one() << index.bit_index;
    }
    
    pub fn clear_bit(&mut self, index: BitMapIndex<T>) where T: BitAndAssign {
        assert!(index.word_index < N);

        self.data[index.word_index] &= !(T::one() << index.bit_index);
    }

    pub fn iter(&self) -> BitMapIterator<'_, T, N> {
        self.into_iter()
    }

    pub fn enumerate(&self) -> impl Iterator<Item=(BitMapIndex<T>, bool)> {
        self.into_iter()
            .enumerate()
            .map(|(index, value)| (BitMapIndex::new(index), value))
    }
}

impl<T: PrimInt, const N: usize> Default for BitMap<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BitMapIterator<'a, T, const N: usize> {
    index: BitMapIndex<T>,
    bitmap: &'a BitMap<T, N>
}

impl<T: PrimInt, const N: usize> Iterator for BitMapIterator<'_, T, N> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let bits_per_word = core::mem::size_of::<T>() * 8;

        if self.index.word_index >= N {
            return None;
        }

        let bit = self.bitmap.index(self.index);
    
        self.index.bit_index += 1;
        if self.index.bit_index >= bits_per_word {
            self.index.bit_index = 0;
            self.index.word_index += 1;
        }

        Some(bit)
    }
}

impl<'a, T: PrimInt, const N: usize> IntoIterator for &'a BitMap<T, N> {
    type Item = bool;
    type IntoIter = BitMapIterator<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        BitMapIterator {
            bitmap: self,
            index: BitMapIndex::new(0)
        }
    }
}