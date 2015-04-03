extern crate byteorder;
extern crate num;

use std;
use std::cmp::*;
use std::clone::*;
use std::hash::*;
use std::collections::*;

use self::byteorder::*;

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
        BitVec::from_bytes(&[different]).iter()
            .enumerate()
            .filter(|&(_, b)| b)
            .map(|(i, _)| i)
            .collect()
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
        let mut buf = vec![0; 64usize];
        <LittleEndian as ByteOrder>::write_u64(&mut buf, different as u64);

        BitVec::from_bytes(&buf[..]).iter()
            .enumerate()
            .filter(|&(_, b)| b)
            .map(|(i, _)| i)
            .collect()
    }
}

impl Hamming for usize {
    // `count_ones` should be faster than iterating over vectors as would happen
    // with the default implementation
    //
    fn hamming(&self, other: &usize) -> usize {
        (*self ^ *other).count_ones() as usize // bitxor
    }
    fn hamming_indices(&self, other: &usize) -> Vec<usize> {
        let different = *self ^ *other;
        let mut buf = vec![0; std::usize::BYTES as usize];
        <LittleEndian as ByteOrder>::write_u64(&mut buf, different as u64);

        BitVec::from_bytes(&buf[..]).iter()
            .enumerate()
            .filter(|&(_, b)| b)
            .map(|(i, _)| i)
            .collect()
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


    // USIZE tests

    #[test]
    fn test_hamming_zero_usize() {
        let a = 0b00000000usize;
        let b = 0b00000000usize;

        assert_eq!(a.hamming(&b), 0);
    }

    #[test]
    fn test_hamming_one_usize() {
        let a = 0b00000000usize;
        let b = 0b10000000usize;
        let c = 0b00000001usize;
        let d = 0b00010000usize;

        assert_eq!(a.hamming(&b), 1);
        assert_eq!(a.hamming(&c), 1);
        assert_eq!(a.hamming(&d), 1);
    }

    #[test]
    fn test_hamming_max_usize() {
        let a = 0b00000000usize;
        let b = 0b11111111usize;

        assert_eq!(a.hamming(&b), 8);
    }
}
