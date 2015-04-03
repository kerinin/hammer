use std;
use std::clone;
use std::iter;

use db::DeletionVariant;

pub struct DeletionVariantIter<T> {
    // The original value, which shouldn't be modified
    source: T,
    // Mutable clone of `original`, returned from `next` as deletion variant
    variant: T,
    // Iteration cursor
    index: usize,
    // The number of dimensions to iterate over
    dimensions: usize,
}

impl<T> DeletionVariantIter<T> where T: clone::Clone {
    pub fn new(v: T, dimensions: usize) -> Self {
        DeletionVariantIter {
            variant: v.clone(),
            source: v,
            index: 0,
            dimensions: dimensions,
        }
    }
}

impl<T> DeletionVariant for T
where T: clone::Clone,
    DeletionVariantIter<T>: Iterator,
{
    type Iter = DeletionVariantIter<T>;

    fn deletion_variants(&self, dimensions: usize) -> DeletionVariantIter<T> {
        DeletionVariantIter::new(self.clone(), dimensions)
    }
}

impl iter::Iterator for DeletionVariantIter<u8> {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<(u8, u8)> {
        if self.index >= self.dimensions {
            None
        } else {
            let next_value = (self.source.clone() | (1u8 << self.index), self.index as u8);
            self.index += 1;
            Some(next_value)
        }
    }
}

impl iter::Iterator for DeletionVariantIter<usize> {
    type Item = (usize, u32);

    fn next(&mut self) -> Option<(usize, u32)> {
        if self.index >= self.dimensions {
            None
        } else {
            let next_value = (self.source.clone() | (1usize << self.index), self.index as u32);
            self.index += 1;
            Some(next_value)
        }
    }
}

impl iter::Iterator for DeletionVariantIter<Vec<u8>> {
    type Item = (Vec<u8>, u32);

    fn next(&mut self) -> Option<(Vec<u8>, u32)> {
        if self.index >= self.dimensions {
            None
        } else {
            if self.index > 0 {
                self.variant[self.index - 1] = self.source[self.index - 1];
            }
            self.variant[self.index] = std::u8::MAX;
            self.index += 1;

            Some((self.variant.clone(), (self.index - 1) as u32))
        }
    }
}

#[cfg(test)] 
mod test {
    use db::*;

    #[test]
    fn test_deletion_variants_vec_u8() {
        let a = vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
        let expected = vec![
            (vec![255u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8], 0u32),
            (vec![0u8, 255u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8], 1u32),
            (vec![0u8, 0u8, 255u8, 0u8, 0u8, 0u8, 0u8, 0u8], 2u32),
            (vec![0u8, 0u8, 0u8, 255u8, 0u8, 0u8, 0u8, 0u8], 3u32),
        ];

        assert_eq!(a.deletion_variants(4).collect::<Vec<(Vec<u8>, u32)>>(), expected);
    }

    #[test]
    fn test_deletion_variants_u8() {
        let a = 0b00000000u8;
        let expected = vec![
            (0b00000001u8, 0u8),
            (0b00000010u8, 1u8),
            (0b00000100u8, 2u8),
            (0b00001000u8, 3u8),
        ];

        assert_eq!(a.deletion_variants(4).collect::<Vec<(u8, u8)>>(), expected);
    }

    #[test]
    fn test_deletion_variants_usize() {
        let a = 0b00000000usize;
        let expected = vec![
            (0b00000001usize, 0u32),
            (0b00000010usize, 1u32),
            (0b00000100usize, 2u32),
            (0b00001000usize, 3u32),
        ];

        assert_eq!(a.deletion_variants(4).collect::<Vec<(usize, u32)>>(), expected);
    }
}
