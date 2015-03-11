use std;
use std::cmp;
use std::clone;
use std::hash;
use std::fmt;
use std::iter;
use std::collections::BitVec;

use std::num::Int;

use db::permutable::Permutable;

//pub trait Value: Permutable + cmp::Eq + hash::Hash + clone::Clone + fmt::Show + iter::FromIterator<u8> {
pub trait Value: Permutable + cmp::Eq + hash::Hash + clone::Clone + fmt::Debug {
    fn transform(&self, shift: u64, mask: u64) -> Self;
    fn permutations(&self, n: u64) -> Vec<Self>;
    fn hamming(&self, rhs: &Self) -> u64;
}

impl Value for Vec<u8> {
    fn transform(&self, shift: u64, mask: u64) -> Vec<u8> {
        let shifted = self.p_shl(&shift);

        let full_byte_count = mask / 8;
        let tail_bits = mask % 8;
        let partial_mask = 0b11111111u8.shl(&(8-tail_bits));

        let mut mask = iter::repeat(0b11111111u8).take(full_byte_count).collect::<Vec<u8>>();
        mask.push(partial_mask);

        shifted.p_bitand(&mask)
    }

    fn permutations(&self, n: u64) -> Vec<Vec<u8>> {
        let bv = BitVec::from_bytes(self.as_slice());

        return range(0u64, n)
            .map(|i| -> Vec<u8> {
                let mut permutation = bv.clone();
                let old_val = permutation.get(i);
                permutation.set(i, !old_val);

                permutation.to_bytes()
            })
        .collect::<Vec<Vec<u8>>>();
    }

    fn hamming(&self, other: &Vec<u8>) -> u64 {
        let shared_bits = self.p_bitxor(other);
        let shared_bitv = BitVec::from_bytes(shared_bits.as_slice());

        return shared_bitv.iter().filter(|x| *x).count();
    }
}

impl Value for u64 {
    fn transform(&self, shift: u64, mask: u64) -> u64 {
        let shifted = self.p_shl(&shift);

        let ones = std::u64::MAX;
        let mask = ones.p_shl(&(std::u64::BITS - mask));

        shifted.p_bitand(&mask)
    }

    fn permutations(&self, n: u64) -> Vec<u64> {
        return range(0u64, n)
            .map(|i| -> u64 {
                let delta = 1u64.p_shr(&i);
                self.clone().bitxor(&delta)
            })
        .collect::<Vec<u64>>();
    }

    fn hamming(&self, other: &u64) -> u64 {
        self.bitxor(other).count_ones()
    }
}
