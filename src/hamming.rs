extern crate num;

use  std::collections::bitv;

use super::permutable::{Permutable};

pub trait Hamming<T> {
    fn hamming(&self, rhs: &T) -> uint;
}

impl Hamming<Vec<u8>> for Vec<u8> {
    // This should really be done by the processor
    fn hamming(&self, other: &Vec<u8>) -> uint {
        let shared_bits = self.bitxor(other);
        let shared_bitv = bitv::from_bytes(shared_bits.as_slice());

        return shared_bitv.iter().filter(|x| *x).count();
    }
}
