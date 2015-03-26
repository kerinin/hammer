extern crate byteorder;

use std;
use std::cmp;
use std::clone;
use std::hash;
use std::num::Int;

use std::collections::BitVec;

use db::{Value, Window, SubstitutionVariant, DeletionVariant};
use self::byteorder::{ByteOrder, LittleEndian};

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

impl Window for Vec<u8> {
    fn window(&self, start_dimension: usize, dimensions: usize) -> Vec<u8> {
        self[start_dimension..(start_dimension + dimensions)].to_vec()
    }
}

impl Value for u8 {
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

impl Value for usize {
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

        BitVec::from_bytes(buf.as_slice()).iter()
            .enumerate()
            .filter(|&(_, b)| b)
            .map(|(i, _)| i)
            .collect()
    }
}

impl<T: cmp::Eq + clone::Clone + hash::Hash> Value for Vec<T> {
    fn hamming_indices(&self, other: &Vec<T>) -> Vec<usize> {
        self.iter()
            .zip(other.iter())
            .enumerate()
            .filter(|&(_, (self_i, other_i))| self_i != other_i)
            .map(|(i, _)| i)
            .collect()
    }
}

impl Value for (u8, u8) {
    fn hamming(&self, other: &(u8, u8)) -> usize {
        let &(self_value, self_deleted_index) = self;
        let &(other_value, other_deleted_index) = other;

        let deletion_different = (1u8 << self_deleted_index) ^ (1u8 << other_deleted_index);
        let binary_different = self_value ^ other_value;

        return (deletion_different | binary_different).count_ones() as usize;
    }

    fn hamming_indices(&self, other: &(u8, u8)) -> Vec<usize> {
        let &(self_value, self_deleted_index) = self;
        let &(other_value, other_deleted_index) = other;

        let deletion_different = (1u8 << self_deleted_index) ^ (1u8 << other_deleted_index);
        let binary_different = self_value ^ other_value;

        let different = deletion_different | binary_different;

        BitVec::from_bytes(&[different]).iter()
            .enumerate()
            .filter(|&(_, b)| b)
            .map(|(i, _)| i)
            .collect()
    }
}

impl Value for (usize, u8) {
    fn hamming(&self, other: &(usize, u8)) -> usize {
        let &(self_value, self_deleted_index) = self;
        let &(other_value, other_deleted_index) = other;

        let deletion_different = (1usize << self_deleted_index) ^ (1usize << other_deleted_index);
        let binary_different = self_value ^ other_value;

        return (deletion_different | binary_different).count_ones() as usize;
    }

    fn hamming_indices(&self, other: &(usize, u8)) -> Vec<usize> {
        let &(self_value, self_deleted_index) = self;
        let &(other_value, other_deleted_index) = other;

        let deletion_different = (1usize << self_deleted_index) ^ (1usize << other_deleted_index);
        let binary_different = self_value ^ other_value;

        let different = deletion_different | binary_different;
        let mut buf = vec![0; std::usize::BYTES as usize];
        // NOTE: This may be doing wierd stuff on 32-bit systems
        <LittleEndian as ByteOrder>::write_u64(&mut buf, different as u64);

        BitVec::from_bytes(buf.as_slice()).iter()
            .enumerate()
            .filter(|&(_, b)| b)
            .map(|(i, _)| i)
            .collect()
    }
}

impl<T: cmp::Eq + clone::Clone + hash::Hash> Value for (Vec<T>, usize) {
    fn hamming_lte(&self, other: &(Vec<T>, usize), bound: usize) -> bool {
        let mut hamming = 0;
        let &(ref self_value, self_deleted_index) = self;
        let &(ref other_value, other_deleted_index) = other;

        for i in (0..self_value.len()) {
            let different = (self_deleted_index == i && other_deleted_index != i) ||
                (self_deleted_index != i && other_deleted_index == i) ||
                (self_deleted_index != i && other_deleted_index != i && self_value[i] != other_value[i]);

            if different {
                hamming += 1;
                if hamming >= bound {
                    return true;
                }
            }
        }

        return false;
    }

    fn hamming_indices(&self, other: &(Vec<T>, usize)) -> Vec<usize> {
        let &(ref self_value, self_deleted_index) = self;
        let &(ref other_value, other_deleted_index) = other;

        (0..self_value.len()).filter(|&i| {
            (self_deleted_index == i && other_deleted_index != i) ||
            (self_deleted_index != i && other_deleted_index == i) ||
            (self_deleted_index != i && other_deleted_index != i && self_value[i] != other_value[i])
        }).collect()
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

impl DeletionVariant for Vec<u8> {
    type Output = (Vec<u8>, usize);

    fn deletion_variants(&self, dimensions: usize) -> Vec<(Vec<u8>, usize)> {
        return range(0, dimensions)
            .map(|i| {
                let mut cloned_self = self.clone();
                cloned_self[i] = std::u8::MAX;
                (cloned_self, i)
            })
            .collect::<Vec<(Vec<u8>, usize)>>();
    }
}


#[cfg(test)] 
mod test {
    use db::{Value, Window, SubstitutionVariant, DeletionVariant};

    // Vec<u8> tests
    #[test]
    fn test_window_min_start_and_finish_vec_u8() {
        let a = vec![1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8];
        let b = vec![];

        assert_eq!(a.window(0,0), b);
    }

    #[test]
    fn test_window_max_start_vec_u8() {
        let a = vec![1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8];
        let b = vec![1u8];

        assert_eq!(a.window(7,1), b);
    }

    #[test]
    fn test_window_min_start_and_max_finish_vec_u8() {
        let a = vec![1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8];
        let b = vec![1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8];

        assert_eq!(a.window(0,8), b);
    }

    #[test]
    fn test_window_n_start_and_max_finish_vec_u8() {
        let a = vec![1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8];
        let b = vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8];

        assert_eq!(a.window(1,7), b);
    }

    #[test]
    fn test_window_min_start_and_n_finish_vec_u8() {
        let a = vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8];
        let b = vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];

        assert_eq!(a.window(0,7), b);
    }

    #[test]
    fn test_window_n_start_and_n_finish_vec_u8() {
        let a = vec![0u8, 0u8, 0u8, 1u8, 1u8, 0u8, 0u8, 0u8];
        let b = vec![1u8, 1u8];

        assert_eq!(a.window(3,2), b);
    }

    #[test]
    fn test_deletion_variants_vec_u8() {
        let a = vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
        let expected = vec![
            (vec![255u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8], 0usize),
            (vec![0u8, 255u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8], 1usize),
            (vec![0u8, 0u8, 255u8, 0u8, 0u8, 0u8, 0u8, 0u8], 2usize),
            (vec![0u8, 0u8, 0u8, 255u8, 0u8, 0u8, 0u8, 0u8], 3usize),
        ];

        assert_eq!(a.deletion_variants(4), expected);
    }

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
