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

impl Iterator for BinaryIter<u8> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.index >= self.dimensions {
            None
        } else {
            let next_value = self.source.clone() ^ (1u8 << self.index);
            self.index += 1;
            Some(next_value)
        }
    }
}

impl Iterator for BinaryIter<u64> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.index >= self.dimensions {
            None
        } else {
            let next_value = self.source.clone() ^ (1u64 << self.index);
            self.index += 1;
            Some(next_value)
        }
    }
}

#[cfg(test)] 
mod test {
    use db::substitution_variant::*;

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
}
