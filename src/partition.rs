extern crate num;

use std::collections::{Map, MutableMap};
use std::iter::Repeat;
use std::hash::Hash;
use std::vec::Vec;
use self::num::rational::Ratio;

struct Partition<T> {
    shift: int,
    mask: int,

    kv: T, // #find(&K)->Option(&V), #insert(K,V)->bool, #remove(&K)->bool
}

trait Permutable<RHS, Result> {
    fn bitxor(&self, rhs: &RHS) -> Result;
    fn bitand(&self, rhs: &RHS) -> Result;
    //fn shl(&self, rhs: &RHS) -> Result;
}

impl Permutable<Vec<u8>, Vec<u8>> for Vec<u8> {
    /*
     * Returns the result of bitxor-ing each byte of self and other.
     * If other is shorter than self, 0 will be used, if self is shorter than
     * other, the trailing bytes of other will be ignored
     */
    fn bitxor(&self, other: &Vec<u8>) -> Vec<u8> {
        let zero: &u8 = &0;
        let other_then_zero = other.iter().chain(Repeat::new(zero));

        return self.iter()
            .zip(other_then_zero)
            .map(|(self_byte, other_byte)| self_byte.clone().bitxor(other_byte) )
            .collect::<Vec<u8>>();
    }

    /*
     * Returns the result of bitand-ing each byte of self and other.
     * If other is shorter than self, 0 will be used, if self is shorter than
     * other, the trailing bytes of other will be ignored
     */
    fn bitand(&self, other: &Vec<u8>) -> Vec<u8> {
        let zero: &u8 = &0;
        let other_then_zero = other.iter().chain(Repeat::new(zero));

        return self.iter()
            .zip(other_then_zero)
            .map(|(self_byte, other_byte)| self_byte.clone().bitand(other_byte) )
            .collect::<Vec<u8>>();
    }

    /*
     * Returns a new byte array with RHS bits removed from the left side, and
     * pads the left-most byte with zeros (if necessary)
     */
    fn shl(&self, rhs: &uint) -> Vec<u8> {
        let to_drop = rhs / 8;
        let to_shift = rhs * 8;
        // We want to drop `to_drop` elements from the vector, 
        // and shift the remaining elements to the left by `to_shift` steps,
        // backfilling with values from the next element
        let mut out = self.skip(to_drop).peekable();
        loop {
            match out.peek() {
                None => {
                    let mut bytes = out.next();
                    bytes.shl(to_shift);
                },
                Some(next_byte) => {
                    let next_bytes_bits = next_byte.Shr(8 - to_shift);
                    let mut bytes = out.next();
                    bytes.shl(to_shift).BitOr(next_bytes_bits);
                },
            }
        }
        return out;
    }
}

/*
impl<T: Map<Vec<u8>, Vec<u8>> + MutableMap<Vec<u8>, Vec<u8>>> Partition<T> {
    pub fn find(&self, key: Vec<u8>) -> Vec<Vec<u8>> {
        let transformed_key = self.transform_key(key);
        let permutations = self.permute_key(transformed_key);

        return permutations
            .map(|&k| self.kv.find(k))
            .reject(|&k| k == None);
    }

    pub fn insert(&mut self, key: Vec<u8>) -> bool {
        let transformed_key = self.transform_key(key);
        let permutations = self.permute_key(transformed_key);

        return permutations
            .map(|&k| self.kv.insert(k, key))
            .all_true;  // Probably not really a function
    }

    pub fn remove(&mut self, key: Vec<u8>) -> bool {
        let transformed_key = self.transform_key(key);
        let permutations = self.permute_key(transformed_key);

        return permutations
            .map(|&k| self.kv.remove(k))
            .all_true;  // Probably not really a function
    }

    /*
     * Transform the full key into this partition's keyspace.  Generally involves
     * dropping all but a fraction of the data
     */
    fn transform_key(&self, key: Vec<u8>) -> Vec<u8> {
        let shifted = key.shl(self.shift);
        let mask = self.mask as Vec<u8>;
        let masked = shifted & mask;
        let byte_count = Ratio::new(self.shift + self.mask, 8).ceil();
        
        return masked.take(byte_count);
    }

    /*
     * Returns an array containing all possible binary 1-permutations of the key
     */
    fn permute_key(&self, key: Vec<u8>) -> Vec<Vec<u8>> {
        let byte_count = Ratio::new(self.shift + self.mask, 8).ceil();
        let mut mask: Vec<u8> = Vec::with_capacity(byte_count);
        mask[0] = mask[0] & 1;

        return range(0,self.mask).map(|&i| key ^ 1.shr(i))
    }
}
*/

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use super::{Permutable};

    #[test]
    fn the_truth() {
        assert!(true);
    }

    #[test]
    fn bitxor_equally_sized_vectors() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let c = vec![0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8];

        assert_eq!(a.bitxor(&b), c);
    }

    #[test]
    fn bitxor_left_vector_longer() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let c = vec![0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];

        let a = vec![0b11111111u8, 0b00000000u8];
        let b = vec![];
        let c = vec![0b11111111u8, 0b00000000u8];

        assert_eq!(a.bitxor(&b), c);
    }

    #[test]
    fn bitxor_right_vector_longer() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let c = vec![0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8];

        assert_eq!(a.bitxor(&b), c);
    }

    #[test]
    fn bitand_equally_sized_vectors() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let c = vec![0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8];

        assert_eq!(a.bitand(&b), c);
    }

    #[test]
    fn bitand_left_vector_longer() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let c = vec![0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8, 0b00000000u8, 0b00000000u8];

        assert_eq!(a.bitand(&b), c);
    }

    #[test]
    fn bitand_right_vector_longer() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let c = vec![0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8];

        assert_eq!(a.bitand(&b), c);
    }

    /*
     *
     * IE, if self =    [00000000] [11111111] [00000000] [11111111]
     *        RHS = 10                ^ new start
     *        Result =  [11111100] [00000011] [11111100]
     */
    #[test]
    fn shl_less_than_vector_length() {
        let a = vec![0b00000000u8, 0b11111111u8];
        let b = vec![0b00001111u8, 0b11110000u8];

        assert_eq!(a.shl(4), b);
    }

    #[test]
    fn shl_more_than_vector_length() {
    }
}
