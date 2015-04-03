use std::clone;
use std::iter;

/// Return a set of single-dimensional permutation variants
///
pub trait SubstitutionVariant {
    type Iter: Iterator<Item = Self>;

    /// Substitution variants
    ///
    /// Returns an array of all possible single-column permutation of `self`.
    /// Alternately, returns the set of values with Hamming distance `1` from 
    /// `self`
    ///
    fn substitution_variants(&self, dimensions: usize) -> <Self as SubstitutionVariant>::Iter;
}

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

impl<T> BinarySubstitutionVariantIter<T>
where T: clone::Clone,
{
    pub fn new(v: T, dimensions: usize) -> Self {
        BinarySubstitutionVariantIter {
            // variant: v.clone(),
            source: v,
            index: 0,
            dimensions: dimensions,
        }
    }
}

impl<T> SubstitutionVariant for T
where T: clone::Clone,
    BinarySubstitutionVariantIter<T>: Iterator<Item = T>,
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

impl iter::Iterator for BinarySubstitutionVariantIter<u64> {
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
