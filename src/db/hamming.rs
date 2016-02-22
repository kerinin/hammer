extern crate byteorder;
extern crate num;

use std::cmp::*;
use std::clone::*;
use std::hash::*;

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

impl Hamming for u8 {
    // `count_ones` should be faster than iterating over vectors as would happen
    // with the default implementation
    //
    fn hamming(&self, other: &u8) -> usize {
        (*self ^ *other).count_ones() as usize // bitxor
    }
    fn hamming_indices(&self, other: &u8) -> Vec<usize> {
        let different = *self ^ *other;

        (0..8).filter(|i| 0u8 != 1u8 << i & different ).collect()
    }
}

impl Hamming for u64 {
    // `count_ones` should be faster than iterating over vectors as would happen
    // with the default implementation
    //
    fn hamming(&self, other: &u64) -> usize {
        (*self ^ *other).count_ones() as usize // bitxor
    }
    fn hamming_indices(&self, other: &u64) -> Vec<usize> {
        let different = *self ^ *other;

        (0..64).filter(|i| 0u64 != 1u64 << i & different ).collect()
    }
}

// Ignoring the deletion index for now
impl Hamming for (u8, u8) {
    fn hamming(&self, other: &(u8, u8)) -> usize {
        let &(self_value, _) = self;
        let &(ref other_value, _) = other;
        self_value.hamming(other_value)
    }

    fn hamming_indices(&self, other: &(u8, u8)) -> Vec<usize> {
        let &(self_value, _) = self;
        let &(ref other_value, _) = other;
        self_value.hamming_indices(other_value)
    }
}

// Ignoring the deletion index for now
impl Hamming for (u64, u8) {
    fn hamming(&self, other: &(u64, u8)) -> usize {
        let &(self_value, _) = self;
        let &(ref other_value, _) = other;
        self_value.hamming(other_value)
    }

    fn hamming_indices(&self, other: &(u64, u8)) -> Vec<usize> {
        let &(self_value, _) = self;
        let &(ref other_value, _) = other;
        self_value.hamming_indices(other_value)
    }
}

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
