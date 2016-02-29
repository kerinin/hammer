use std::mem::size_of;
use std::clone::Clone;
use std::iter::Iterator;

/// Iterator for values that can be treated as a bitmap
///
pub struct BinaryIter<T> {
    // The original value, which shouldn't be modified
    source: T,
    // Iteration cursor
    index: usize,
    // The number of dimensions to iterate over
    dimensions: usize,
}

impl<T> BinaryIter<T>
where T: Clone,
{
    pub fn new(v: T, dimensions: usize) -> Self {
        BinaryIter {
            source: v,
            index: 0,
            dimensions: dimensions,
        }
    }
}

macro_rules! binary_iterator {
    ($elem:ident) => {
        impl Iterator for BinaryIter<$elem> {
            type Item = $elem;

            fn next(&mut self) -> Option<$elem> {
                if self.index >= self.dimensions {
                    None
                } else {
                    let next_value = self.source.clone() ^ (1 << self.index);
                    self.index += 1;
                    Some(next_value)
                }
            }
        }
    }
}
binary_iterator!(u8);
binary_iterator!(u16);
binary_iterator!(u32);
binary_iterator!(u64);

macro_rules! binary_array_iterator {
    ([$elem:ty; $elems:expr]) => {
        impl Iterator for BinaryIter<[$elem; $elems]> {
            type Item = [$elem; $elems];

            fn next(&mut self) -> Option<[$elem; $elems]> {
                let offset = self.index / (8 * size_of::<$elem>());
                let index = self.index % (8 * size_of::<$elem>());

                if self.index >= self.dimensions {
                    None
                } else {
                    let mut next_value = self.source.clone();
                    next_value[offset] = next_value[offset] ^ (1 << index);
                    self.index += 1;
                    Some(next_value)
                }
            }
        }
    }
}
binary_array_iterator!([u64; 4]);
binary_array_iterator!([u64; 2]);

#[cfg(test)] 
mod test {
    use db::substitution::{SubstitutionVariant};

    #[test]
    fn test_substitution_variants_u8() {
        let a = 0b00000000u8;
        let expected = vec![
            0b00000001u8,
            0b00000010u8,
            0b00000100u8,
            0b00001000u8,
        ];

        assert_eq!(a.substitution_variants(4).collect::<Vec<u8>>(), expected);
    }

    #[test]
    fn test_substitution_variants_u64() {
        let a = 0b00000000u64;
        let expected = vec![
            0b00000001u64,
            0b00000010u64,
            0b00000100u64,
            0b00001000u64,
        ];

        assert_eq!(a.substitution_variants(4).collect::<Vec<u64>>(), expected);
    }

    #[test]
    fn test_substitution_variants_u64x2() {
        let a = [0b00000000u64, 0b00000000u64];
        let expected = vec![
            [0b00000001u64, 0b00000000u64],
            [0b00000010u64, 0b00000000u64],
            [0b00000100u64, 0b00000000u64],
                [0b00001000u64, 0b00000000u64],
        ];

        assert_eq!(a.substitution_variants(4).collect::<Vec<[u64; 2]>>(), expected);
    }
}
