use std;
use std::cmp;
use std::clone;
use std::hash;
use std::fmt;
use std::iter;
use std::collections::bitv;

use std::num::Int;

use db::permutable::Permutable;

//pub trait Value: Permutable + cmp::Eq + hash::Hash + clone::Clone + fmt::Show + iter::FromIterator<u8> {
pub trait Value: Permutable + cmp::Eq + hash::Hash + clone::Clone + fmt::Show {
    fn transform(&self, shift: uint, mask: uint) -> Self;
    fn permutations(&self, n: uint) -> Vec<Self>;
    fn hamming(&self, rhs: &Self) -> uint;
}

impl Value for Vec<u8> {
    fn transform(&self, shift: uint, mask: uint) -> Vec<u8> {
        let shifted = self.p_shl(&shift);

        let full_byte_count = mask / 8;
        let tail_bits = mask % 8;
        let partial_mask = 0b11111111u8.shl(&(8-tail_bits));

        let mut mask = iter::repeat(0b11111111u8).take(full_byte_count).collect::<Vec<u8>>();
        mask.push(partial_mask);

        shifted.p_bitand(&mask)
    }

    fn permutations(&self, n: uint) -> Vec<Vec<u8>> {
        let bv = bitv::from_bytes(self.as_slice());

        return range(0u, n)
            .map(|i| -> Vec<u8> {
                let mut permutation = bv.clone();
                let old_val = permutation.get(i);
                permutation.set(i, !old_val);

                permutation.to_bytes()
            })
        .collect::<Vec<Vec<u8>>>();
    }

    fn hamming(&self, other: &Vec<u8>) -> uint {
        let shared_bits = self.p_bitxor(other);
        let shared_bitv = bitv::from_bytes(shared_bits.as_slice());

        return shared_bitv.iter().filter(|x| *x).count();
    }
}

impl Value for uint {
    fn transform(&self, shift: uint, mask: uint) -> uint {
        let shifted = self.p_shl(&shift);

        let ones = std::uint::MAX;
        let mask = ones.p_shl(&(std::uint::BITS - mask));

        shifted.p_bitand(&mask)
    }

    fn permutations(&self, n: uint) -> Vec<uint> {
        return range(0u, n)
            .map(|i| -> uint {
                let delta = 1u.p_shr(&i);
                self.clone().bitxor(&delta)
            })
        .collect::<Vec<uint>>();
    }

    fn hamming(&self, other: &uint) -> uint {
        self.bitxor(other).count_ones()
    }
}
