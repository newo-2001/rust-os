use core::{marker::PhantomData, ops::{BitAndAssign, BitOrAssign}};

use num_traits::PrimInt;

pub struct BitMap<const N: usize, T> {
    data: [T; N]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl<T: PrimInt> From<usize> for BitMapIndex<T> {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl<T, const N: usize> BitMap<N, T> {
    pub const fn from_data(data: [T; N]) -> Self {
        Self { data }
    }
}

#[expect(clippy::missing_panics_doc)]
impl<T: PrimInt, const N: usize> BitMap<N, T> {
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

    pub fn iter(&self) -> BitMapIterator<'_, N, T> {
        self.into_iter()
    }

    pub fn enumerate(&self) -> impl Iterator<Item=(BitMapIndex<T>, bool)> {
        self.into_iter()
            .enumerate()
            .map(|(index, value)| (BitMapIndex::new(index), value))
    }
}

impl<T: PrimInt, const N: usize> Default for BitMap<N, T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BitMapIterator<'a, const N: usize, T> {
    index: BitMapIndex<T>,
    bitmap: &'a BitMap<N, T>
}

impl<T: PrimInt, const N: usize> Iterator for BitMapIterator<'_, N, T> {
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

impl<'a, T: PrimInt, const N: usize> IntoIterator for &'a BitMap<N, T> {
    type Item = bool;
    type IntoIter = BitMapIterator<'a, N, T>;

    fn into_iter(self) -> Self::IntoIter {
        BitMapIterator {
            bitmap: self,
            index: BitMapIndex::new(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn set_bit_sets_corresponding_bit() {
        let mut map = BitMap::<2, u32>::new();
        map.set_bit(BitMapIndex::new(35));
        assert_eq!([0, (1 << 3)], map.data);
    }   
        
    #[test]
    fn clear_bit_clears_corresponding_bit() {
        let mut map = BitMap::<2, u8>::from_data([0xff; 2]);
        map.clear_bit(BitMapIndex::new(12));
        assert_eq!([0xff, !(1 << 4)], map.data);
    }

    #[test]
    fn write_bit_true_sets_bit() {
        const BYTE: u8 = 0b10101010;
        let mut map = BitMap::<2, u8>::from_data([BYTE; 2]);
        map.write_bit(BitMapIndex::new(10), true);
        assert_eq!([BYTE, 0b10101110], map.data);
    }

    #[test]
    fn write_bit_false_clears_bit() {
        const BYTE: u8 = 0b10101010;
        let mut map = BitMap::<2, u8>::from_data([BYTE; 2]);
        map.write_bit(BitMapIndex::new(9), false);
        assert_eq!([BYTE, 0b10101000], map.data);
    }

    #[test]
    fn index_in_range_returns_bit_at_index() {
        let map = BitMap::<2, u8>::from_data([0, 0b00100000]);
        let index = BitMapIndex::new(13);

        assert!(map.index(index));
        assert_eq!(Some(true), map.get(index));
    }

    #[test]
    fn get_out_of_range_returns_none() {
        let map = BitMap::<1, u8>::new();
        assert_eq!(None, map.get(BitMapIndex::new(8)));
    }

    #[test]
    fn iter_iterates_all_bits() {
        let map = BitMap::<2, u8>::from_data([0b10100111, 0b10001110]);
        let bits = map.iter().collect_array::<16>().unwrap();
        assert_eq!([
            true, true, true, false, false, true, false, true,
            false, true, true, true, false, false, false, true
        ], bits);
    }

    #[test]
    fn enumerate_enumerates_all_bits() {
        let map = BitMap::<2, u8>::from_data([0b11101001, 0b00100101]);
        let bits = map.enumerate().collect_array::<16>().unwrap();

        let expected = (0..16).map(|index| BitMapIndex {
            word_index: index / 8,
            bit_index: index % 8,
            _word_type: PhantomData::<u8>
        }).zip([
            true, false, false, true, false, true, true, true,
            true, false, true, false, false, true, false, false
        ]).collect_array::<16>().unwrap();

        assert_eq!(expected, bits)
    }
}