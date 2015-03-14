use std;
use std::hash;
use std::cmp;
use std::clone;
use std::ops;
use std::num::Int;

//pub trait Value: Permutable + cmp::Eq + hash::Hash + clone::Clone + fmt::Show + iter::FromIterator<u8> {
pub trait Value: hash::Hash + cmp::Eq + clone::Clone + ops::BitXor + ops::BitAnd + ops::Shl<usize> {
    fn transform(&self, shift: usize, mask: usize) -> Self;
    fn permutations(&self, n: usize) -> Vec<Self>;
    fn hamming(&self, rhs: &Self) -> usize;
}

impl Value for usize {
    fn transform(&self, shift: usize, mask: usize) -> usize {
        let shifted = self << shift;

        let ones = std::usize::MAX;
        let mask = ones << (std::usize::BITS as usize - mask);

        shifted & mask
    }

    fn permutations(&self, n: usize) -> Vec<usize> {
        return range(0usize, n)
            .map(|i| -> usize {
                let delta = 1usize >> i;
                self.clone() ^ delta // bitxor
            })
        .collect::<Vec<usize>>();
    }

    fn hamming(&self, other: &usize) -> usize {
        (*self ^ *other).count_ones() as usize // bitxor
    }
}
