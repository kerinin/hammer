extern crate byteorder;
extern crate num;

use std::cmp::*;
use std::clone::*;
use std::hash::*;

use db::deletion::{Du8, Du16, Du32, Du64};

/// HmSearch-indexable value
///
pub trait Hamming {
    /// Hamming distance betwen `self` and `rhs`
    ///
    fn hamming(&self, rhs: &Self) -> usize {
        self.hamming_indices(rhs).len()
    }

    /// Returns true if the hamming distance between `self` and `rhs` is less than
    /// or equal to `bound`, false otherwise
    ///
    fn hamming_lte(&self, rhs: &Self, bound: usize) -> bool {
        self.hamming(rhs) <= bound
    }

    /// Returns a vector of dimension indices whose value is different between 
    /// `self` and `rhs`
    ///
    fn hamming_indices(&self, rhs: &Self) -> Vec<usize>;
}

macro_rules! intrinsic_hamming {
    ($elem:ident) => {
        impl Hamming for $elem {
            // `count_ones` should be faster than iterating over vectors as would happen
            // with the default implementation
            //
            fn hamming(&self, other: &$elem) -> usize {
                (*self ^ *other).count_ones() as usize // bitxor
            }
            fn hamming_indices(&self, other: &$elem) -> Vec<usize> {
                let different = *self ^ *other;

                (0..8).filter(|i| (0 as $elem) != (1 as $elem) << i & different ).collect()
            }
        }
    }
}
intrinsic_hamming!(u8);
intrinsic_hamming!(u16);
intrinsic_hamming!(u32);
intrinsic_hamming!(u64);

macro_rules! array_hamming {
    ($elem:ty) => {
        impl Hamming for $elem {
            // `count_ones` should be faster than iterating over vectors as would happen
            // with the default implementation
            //
            fn hamming(&self, other: &$elem) -> usize {
                self.iter().zip(other.iter()).fold(0, |h, (&a, &b)| { h + a.hamming(&b) })
            }
            fn hamming_indices(&self, other: &$elem) -> Vec<usize> {
                let len = self.len();

                self.iter().zip(other.iter()).enumerate().fold(Vec::new(), |mut h, (i, (a, b))| {
                    let offset = i * len;
                    let mut pair_indices = a.hamming_indices(b).iter().map(|idx| idx + offset).collect();
                    h.append(&mut pair_indices);
                    h
                })
            }
        }
    }
}
array_hamming!([u64; 2]);
array_hamming!([u64; 4]);

macro_rules! intrinsic_deletion_hamming {
    ($elem:ident) => {
        // Ignoring the deletion index for now
        impl Hamming for $elem {
            fn hamming(&self, other: &$elem) -> usize {
                let &(self_value, _) = self;
                let &(ref other_value, _) = other;
                self_value.hamming(other_value)
            }

            fn hamming_indices(&self, other: &$elem) -> Vec<usize> {
                let &(self_value, _) = self;
                let &(ref other_value, _) = other;
                self_value.hamming_indices(other_value)
            }
        }
    }
}
intrinsic_deletion_hamming!(Du8);
intrinsic_deletion_hamming!(Du16);
intrinsic_deletion_hamming!(Du32);
intrinsic_deletion_hamming!(Du64);

impl<T: Eq + Clone + Hash> Hamming for Vec<T> {
    // NOTE: Optimize the bound query

    fn hamming_indices(&self, other: &Vec<T>) -> Vec<usize> {
        self.iter()
            .zip(other.iter())
            .enumerate()
            .filter(|&(_, (self_i, other_i))| self_i != other_i)
            .map(|(i, _)| i)
            .collect()
    }
}


#[cfg(test)] 
mod test {
    use db::hamming::*;

    // Vec<u8> tests

    #[test]
    fn test_hamming_zero_vec_u8() {
        let a = vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
        let b = vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];

        assert_eq!(a.hamming(&b), 0);
    }

    #[test]
    fn test_hamming_one_vec_u8() {
        let a = vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
        let b = vec![1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
        let c = vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8];
        let d = vec![0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 0u8];

        assert_eq!(a.hamming(&b), 1);
        assert_eq!(a.hamming(&c), 1);
        assert_eq!(a.hamming(&d), 1);
    }

    #[test]
    fn test_hamming_max_vec_u8() {
        let a = vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
        let b = vec![1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8];

        assert_eq!(a.hamming(&b), 8);
    }


    // u8 tests
    #[test]
    fn test_hamming_zero_u8() {
        let a = 0b00000000u8;
        let b = 0b00000000u8;

        assert_eq!(a.hamming(&b), 0);
    }

    #[test]
    fn test_hamming_one_u8() {
        let a = 0b00000000u8;
        let b = 0b10000000u8;
        let c = 0b00000001u8;
        let d = 0b00010000u8;

        assert_eq!(a.hamming(&b), 1);
        assert_eq!(a.hamming(&c), 1);
        assert_eq!(a.hamming(&d), 1);
    }

    #[test]
    fn test_hamming_max_u8() {
        let a = 0b00000000u8;
        let b = 0b11111111u8;

        assert_eq!(a.hamming(&b), 8);
    }


    // u64 tests

    #[test]
    fn test_hamming_zero_u64() {
        let a = 0b00000000u64;
        let b = 0b00000000u64;

        assert_eq!(a.hamming(&b), 0);
    }

    #[test]
    fn test_hamming_one_u64() {
        let a = 0b00000000u64;
        let b = 0b10000000u64;
        let c = 0b00000001u64;
        let d = 0b00010000u64;

        assert_eq!(a.hamming(&b), 1);
        assert_eq!(a.hamming(&c), 1);
        assert_eq!(a.hamming(&d), 1);
    }

    #[test]
    fn test_hamming_max_u64() {
        let a = 0b00000000u64;
        let b = 0b11111111u64;

        assert_eq!(a.hamming(&b), 8);
    }
}
