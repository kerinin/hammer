use std::clone;
use std::iter;

use db::SubstitutionVariant;

/// Iterator for values that can be treated as a bitmap
///
pub struct BinarySubstitutionVariantIter<T> {
    // The original value, which shouldn't be modified
    source: T,
    // Mutable clone of `original`, returned from `next` as substitution variant
    // variant: T,
    // Iteration cursor
    index: usize,
    // The number of dimensions to iterate over
    dimensions: usize,
}

impl<T> BinarySubstitutionVariantIter<T> where T: clone::Clone {
    pub fn new(v: T, dimensions: usize) -> Self {
        BinarySubstitutionVariantIter {
            // variant: v.clone(),
            source: v,
            index: 0,
            dimensions: dimensions,
        }
    }
}

impl<T> SubstitutionVariant for T where
    T: clone::Clone,
    BinarySubstitutionVariantIter<T>: Iterator<Item = T>
{
    type Iter = BinarySubstitutionVariantIter<T>;

    fn substitution_variants(&self, dimensions: usize) -> BinarySubstitutionVariantIter<T> {
        BinarySubstitutionVariantIter::new(self.clone(), dimensions)
    }
}

impl iter::Iterator for BinarySubstitutionVariantIter<u8> {
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

impl iter::Iterator for BinarySubstitutionVariantIter<usize> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.index >= self.dimensions {
            None
        } else {
            let next_value = self.source.clone() ^ (1usize << self.index);
            self.index += 1;
            Some(next_value)
        }
    }
}
