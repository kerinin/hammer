#[macro_use]

use std;
use std::hash;
use std::cmp;
use std::clone;
use std::num::Int;

// pub trait Value: hash::Hash + cmp::Eq + clone::Clone + ops::BitXor + ops::BitAnd + ops::Shl<usize> + fmt::Debug + fmt::Binary {
pub trait Value: hash::Hash + cmp::Eq + clone::Clone {}

pub trait Window {
    /* 
     * `start_dimension` the index of the 1st dimension to include in the slice, 
     *      0-indexed from least significant
     * `dimensions` the total number of dimensions to include
     */
    fn window(&self, start_dimension: usize, dimensions: usize) -> Self;
}

pub trait SubstitutionVariant where Self: Hamming {
    fn substitution_variants(&self, dimensions: usize) -> Vec<Self>;
}

pub trait DeletionVariant where <Self as DeletionVariant>::Output: Value + Hamming {
    type Output;
    fn deletion_variants(&self, dimensions: usize) -> Vec<<Self as DeletionVariant>::Output>;
}

pub trait Hamming {
    fn hamming(&self, rhs: &Self) -> usize;
    fn hamming_lte(&self, rhs: &Self, bound: usize) -> bool;
}


impl Value for u8 {}
impl Value for usize {}
impl Value for (u8, u8) {}
impl Value for (usize, u8) {}

impl Window for u8 {
    fn window(&self, start_dimension: usize, dimensions: usize) -> u8 {
        //  2/4        11111111
        //              ^<--^
        //  << 1       11111110
        //  >> 1+2     00011111
        let bits = std::u8::BITS as usize;
        let trim_high = bits - (start_dimension + dimensions);
        (self << trim_high) >> (trim_high + start_dimension)
    }
}
impl Window for usize {
    fn window(&self, start_dimension: usize, dimensions: usize) -> usize {
        let bits = std::usize::BITS as usize;
        let trim_high = bits - (start_dimension + dimensions);
        (self << trim_high) >> (trim_high + start_dimension)
    }
}


impl SubstitutionVariant for u8 {
    fn substitution_variants(&self, dimensions: usize) -> Vec<u8> {
        return range(0, dimensions)
            .map(|i| {
                let delta = 1u8 << i;
                self.clone() ^ delta
            })
            .collect::<Vec<u8>>();
    }
}
impl SubstitutionVariant for usize {
    fn substitution_variants(&self, dimensions: usize) -> Vec<usize> {
        return range(0, dimensions)
            .map(|i| {
                let delta = 1usize << i;
                self.clone() ^ delta
            })
            .collect::<Vec<usize>>();
    }
}


impl DeletionVariant for u8 {
    type Output = (u8, u8);

    fn deletion_variants(&self, dimensions: usize) -> Vec<(u8, u8)> {
        return range(0, dimensions)
            .map(|i| {
                (self.clone() | (1u8 << i), i as u8)
            })
            .collect::<Vec<(u8, u8)>>();
    }
}
impl DeletionVariant for usize {
    type Output = (usize, u8);

    fn deletion_variants(&self, dimensions: usize) -> Vec<(usize, u8)> {
        return range(0, dimensions)
            .map(|i| {
                (self.clone() | (1usize << i), i as u8)
            })
            .collect::<Vec<(usize, u8)>>();
    }
}


impl Hamming for u8 {
    fn hamming(&self, other: &u8) -> usize {
        (*self ^ *other).count_ones() as usize // bitxor
    }
    fn hamming_lte(&self, other: &u8, bound: usize) -> bool {
        self.hamming(other) <= bound
    }
}
impl Hamming for usize {
    fn hamming(&self, other: &usize) -> usize {
        (*self ^ *other).count_ones() as usize // bitxor
    }
    fn hamming_lte(&self, other: &usize, bound: usize) -> bool {
        self.hamming(other) <= bound
    }
}
impl Hamming for (u8, u8) {
    fn hamming(&self, other: &(u8, u8)) -> usize {
        let &(self_value, self_deleted_index) = self;
        let &(other_value, other_deleted_index) = other;
        let mask = (1u8 << self_deleted_index) | (1u8 << other_deleted_index);

        let naive_hamming = (self_value | mask) ^ (other_value | mask);

        if self_deleted_index == other_deleted_index {
            return naive_hamming.count_ones() as usize;
        } else {
            return 2 + naive_hamming.count_ones() as usize;
        }
    }

    fn hamming_lte(&self, other: &(u8, u8), bound: usize) -> bool {
        self.hamming(other) <= bound
    }
}
impl Hamming for (usize, u8) {
    fn hamming(&self, other: &(usize, u8)) -> usize {
        let &(self_value, self_deleted_index) = self;
        let &(other_value, other_deleted_index) = other;
        let mask = (1usize << self_deleted_index) | (1usize << other_deleted_index);

        let naive_hamming = (self_value | mask) ^ (other_value | mask);

        if self_deleted_index == other_deleted_index {
            return naive_hamming.count_ones() as usize;
        } else {
            return 2 + naive_hamming.count_ones() as usize;
        }
    }

    fn hamming_lte(&self, other: &(usize, u8), bound: usize) -> bool {
        self.hamming(other) <= bound
    }
}


#[cfg(test)] 
mod test {
    use db::value::{Window, SubstitutionVariant, DeletionVariant, Hamming};
    use db::bit_matrix::BitMatrix;

    #[test]
    fn test_window_min_start_and_finish_u8() {
        let a = 0b10000001u8;
        let b = 0b00000000u8;

        assert_eq!(a.window(0,0), b);
    }

    #[test]
    fn test_window_max_start_u8() {
        let a = 0b10000001u8;
        let b = 0b00000001u8;

        assert_eq!(a.window(7,1), b);
    }

    #[test]
    fn test_window_min_start_and_max_finish_u8() {
        let a = 0b10000001u8;
        let b = 0b10000001u8;

        assert_eq!(a.window(0,8), b);
    }

    #[test]
    fn test_window_n_start_and_max_finish_u8() {
        let a = 0b11000011u8;
        let b = 0b01100001u8;

        assert_eq!(a.window(1,7), b);
    }

    #[test]
    fn test_window_min_start_and_n_finish_u8() {
        let a = 0b11000011u8;
        let b = 0b01000011u8;

        assert_eq!(a.window(0,7), b);
    }

    #[test]
    fn test_window_n_start_and_n_finish_u8() {
        let a = 0b11111000u8;
        let b = 0b00000011u8;

        assert_eq!(a.window(3,2), b);
    }

    #[test]
    fn test_substitution_variants_u8() {
        let a = 0b00000000u8;
        let expected = vec![
            0b00000001u8,
            0b00000010u8,
            0b00000100u8,
            0b00001000u8,
        ];

        assert_eq!(a.substitution_variants(4), expected);
    }
    #[test]

    fn test_deletion_variants_u8() {
        let a = 0b00000000u8;
        let expected = vec![
            (0b00000001u8, 0u8),
            (0b00000010u8, 1u8),
            (0b00000100u8, 2u8),
            (0b00001000u8, 3u8),
        ];

        assert_eq!(a.deletion_variants(4), expected);
    }

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
    fn test_window_min_start_and_finish_usize() {
        let a = 0b10000001usize;
        let b = 0b00000001usize;

        assert_eq!(a.window(0,1), b);
    }

    #[test]
    fn test_window_max_start_usize() {
        let a = 0b10000001usize;
        let b = 0b00000001usize;

        assert_eq!(a.window(7,1), b);
    }

    #[test]
    fn test_window_min_start_and_max_finish_usize() {
        let a = 0b10000001usize;
        let b = 0b10000001usize;

        assert_eq!(a.window(0,8), b);
    }

    #[test]
    fn test_window_n_start_and_max_finish_usize() {
        let a = 0b11000011usize;
        let b = 0b01100001usize;

        assert_eq!(a.window(1,7), b);
    }

    #[test]
    fn test_window_min_start_and_n_finish_usize() {
        let a = 0b11000011usize;
        let b = 0b01000011usize;

        assert_eq!(a.window(0,7), b);
    }

    #[test]
    fn test_window_n_start_and_n_finish_usize() {
        let a = 0b11111000usize;
        let b = 0b00000011usize;

        assert_eq!(a.window(3,2), b);
    }

    #[test]
    fn test_substitution_variants_usize() {
        let a = 0b00000000usize;
        let expected = vec![
            0b00000001usize,
            0b00000010usize,
            0b00000100usize,
            0b00001000usize,
        ];

        assert_eq!(a.substitution_variants(4), expected);
    }

    #[test]
    fn test_deletion_variants_usize() {
        let a = 0b00000000usize;
        let expected = vec![
            (0b00000001usize, 0u8),
            (0b00000010usize, 1u8),
            (0b00000100usize, 2u8),
            (0b00001000usize, 3u8),
        ];

        assert_eq!(a.deletion_variants(4), expected);
    }

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



    #[test]
    fn test_window_min_start_and_finish_bitmatrix() {
        let a = bitmatrix![[0b10000001u8]];
        let b = bitmatrix![[0b00000001u8]];

        assert_eq!(a.window(0,1), b);
    }

    #[test]
    fn test_window_max_start_bitmatrix() {
        let a = bitmatrix![[0b10000001u8]];
        let b = bitmatrix![[0b00000001u8]];

        assert_eq!(a.window(7,1), b);
    }

    #[test]
    fn test_window_min_start_and_max_finish_bitmatrix() {
        let a = bitmatrix![[0b10000001u8]];
        let b = bitmatrix![[0b10000001u8]];

        assert_eq!(a.window(0,8), b);
    }

    #[test]
    fn test_window_n_start_and_max_finish_bitmatrix() {
        let a = bitmatrix![[0b11000011u8]];
        let b = bitmatrix![[0b01100001u8]];

        assert_eq!(a.window(1,7), b);
    }

    #[test]
    fn test_window_min_start_and_n_finish_bitmatrix() {
        let a = bitmatrix![[0b11000011u8]];
        let b = bitmatrix![[0b01000011u8]];

        assert_eq!(a.window(0,7), b);
    }

    #[test]
    fn test_window_n_start_and_n_finish_bitmatrix() {
        let a = bitmatrix![[0b11111000u8]];
        let b = bitmatrix![[0b00000011u8]];

        assert_eq!(a.window(3,2), b);
    }

    #[test]
    fn test_permutation_bitmatrix() {
        let a = bitmatrix![[0b00000000u8]];
        let expected = vec![
            bitmatrix![[0b00000001u8]],
            bitmatrix![[0b00000010u8]],
            bitmatrix![[0b00000100u8]],
            bitmatrix![[0b00001000u8]],
        ];

        assert_eq!(a.substitution_variants(4), expected);
    }

    #[test]
    fn test_hamming_zero_bitmatrix() {
        let a = bitmatrix![[0b00000000u8]];
        let b = bitmatrix![[0b00000000u8]];

        assert_eq!(a.hamming(&b), 0);
    }

    #[test]
    fn test_hamming_one_bitmatrix() {
        let a = bitmatrix![[0b00000000u8]];
        let b = bitmatrix![[0b10000000u8]];
        let c = bitmatrix![[0b00000001u8]];
        let d = bitmatrix![[0b00010000u8]];

        assert_eq!(a.hamming(&b), 1);
        assert_eq!(a.hamming(&c), 1);
        assert_eq!(a.hamming(&d), 1);
    }

    #[test]
    fn test_hamming_max_bitmatrix() {
        let a = bitmatrix![[0b00000000u8]];
        let b = bitmatrix![[0b11111111u8]];

        assert_eq!(a.hamming(&b), 8);
    }

    #[test]
    fn test_deletion_hamming_equal_u8_u8() {
        let a = (0b11111111u8, 0u8);
        let b = (0b11111111u8, 0u8);

        let c = (0b00000001u8, 0u8);
        let d = (0b00000001u8, 0u8);

        assert_eq!(a.hamming(&b), 0);
        assert_eq!(c.hamming(&d), 0);
    }

    #[test]
    fn test_deletion_hamming_binary_unequal_u8_u8() {
        let a = (0b11111111u8, 0u8);
        let b = (0b01111111u8, 0u8);

        let c = (0b10000001u8, 0u8);
        let d = (0b00000001u8, 0u8);

        assert_eq!(a.hamming(&b), 1);
        assert_eq!(c.hamming(&d), 1);
    }

    #[test]
    fn test_deletion_hamming_deleted_unequal_u8_u8() {
        let a = (0b11111111u8, 0u8);
        let b = (0b11111111u8, 1u8);

        let c = (0b00000001u8, 0u8);
        let d = (0b00000010u8, 1u8);

        assert_eq!(a.hamming(&b), 2);
        assert_eq!(c.hamming(&d), 2);
    }

    #[test]
    fn test_deletion_hamming_binary_and_deleted_unequal_u8_u8() {
        let a = (0b11111111u8, 0u8);
        let b = (0b01111111u8, 1u8);

        let c = (0b10000001u8, 0u8);
        let d = (0b00000010u8, 1u8);

        assert_eq!(a.hamming(&b), 3);
        assert_eq!(c.hamming(&d), 3);
    }

    #[test]
    fn test_deletion_hamming_equal_usize_u8() {
        let a = (0b11111111usize, 0u8);
        let b = (0b11111111usize, 0u8);

        let c = (0b00000001usize, 0u8);
        let d = (0b00000001usize, 0u8);

        assert_eq!(a.hamming(&b), 0);
        assert_eq!(c.hamming(&d), 0);
    }

    #[test]
    fn test_deletion_hamming_binary_unequal_usize_u8() {
        let a = (0b11111111usize, 0u8);
        let b = (0b01111111usize, 0u8);

        let c = (0b10000001usize, 0u8);
        let d = (0b00000001usize, 0u8);

        assert_eq!(a.hamming(&b), 1);
        assert_eq!(c.hamming(&d), 1);
    }

    #[test]
    fn test_deletion_hamming_deleted_unequal_usize_u8() {
        let a = (0b11111111usize, 0u8);
        let b = (0b11111111usize, 1u8);

        let c = (0b00000001usize, 0u8);
        let d = (0b00000010usize, 1u8);

        assert_eq!(a.hamming(&b), 2);
        assert_eq!(c.hamming(&d), 2);
    }

    #[test]
    fn test_deletion_hamming_binary_and_deleted_unequal_usize_u8() {
        let a = (0b11111111usize, 0u8);
        let b = (0b01111111usize, 1u8);

        let c = (0b10000001usize, 0u8);
        let d = (0b00000010usize, 1u8);

        assert_eq!(a.hamming(&b), 3);
        assert_eq!(c.hamming(&d), 3);
    }
}
