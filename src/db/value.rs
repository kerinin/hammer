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
    fn transform(&self, shift: usize, mask: usize) -> Self;
    fn permutations(&self, n: usize) -> Vec<Self>;
    fn hamming(&self, rhs: &Self) -> usize;
}

impl Value for Vec<u8> {
    fn transform(&self, shift: usize, mask: usize) -> Vec<u8> {
        let shifted = self.p_shl(&shift);

        let full_byte_count = mask / 8;
        let tail_bits = mask % 8;
        let partial_mask = 0b11111111u8 << (8-tail_bits);

        let mut mask = iter::repeat(0b11111111u8).take(full_byte_count).collect::<Vec<u8>>();
        mask.push(partial_mask);

        shifted.p_bitand(&mask)
    }

    fn permutations(&self, n: usize) -> Vec<Vec<u8>> {
        let bv = BitVec::from_bytes(self.as_slice());

        return range(0usize, n)
            .map(|i| -> Vec<u8> {
                let mut permutation = bv.clone();
                match permutation.get(i) {
                    Some(old_val) => permutation.set(i, !old_val),
                    // NOTE: If more permutations were requested than can be generated, 
                    // we'll just pad the end with unmodified versions
                    _ => () 
                }

                permutation.to_bytes()
            })
        .collect::<Vec<Vec<u8>>>();
    }

    fn hamming(&self, other: &Vec<u8>) -> usize {
        let shared_bits = self.p_bitxor(other);
        let shared_bitv = BitVec::from_bytes(shared_bits.as_slice());

        return shared_bitv.iter().filter(|x| *x).count();
    }
}

impl Value for usize {
    fn transform(&self, shift: usize, mask: usize) -> usize {
        let shifted = self.p_shl(&shift);

        let ones = std::usize::MAX;
        let mask = ones.p_shl(&(std::usize::BITS as usize - mask));

        shifted.p_bitand(&mask)
    }

    fn permutations(&self, n: usize) -> Vec<usize> {
        return range(0usize, n)
            .map(|i| -> usize {
                let delta = 1usize.p_shr(&i);
                self.clone() ^ delta // bitxor
            })
        .collect::<Vec<usize>>();
    }

    fn hamming(&self, other: &usize) -> usize {
        (*self ^ *other).count_ones() as usize // bitxor
    }
}
