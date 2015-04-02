extern crate byteorder;

use std;
use std::cmp;
use std::clone;
use std::hash;
use std::num::Int;

use std::collections::BitVec;

use db::{Value, Window};
use self::byteorder::{ByteOrder, LittleEndian};

impl Window for u8 {
    fn window(&self, start_dimension: usize, dimensions: usize) -> u8 {
        //  2/4        11111111
        //              ^<--^
        //  << 1       11111110
        //  >> 1+2     00011111
        let bits = std::u8::BITS as usize;
        let trim_high = bits - (start_dimension + dimensions);

        if trim_high >= std::u8::BITS as usize {
            0u8
        } else {
            (self << trim_high) >> (trim_high + start_dimension)
        }
    }
}

impl Window for usize {
    fn window(&self, start_dimension: usize, dimensions: usize) -> usize {
        let bits = std::usize::BITS as usize;
        let trim_high = bits - (start_dimension + dimensions);

        if trim_high >= std::usize::BITS as usize {
            0usize
        } else {
            (self << trim_high) >> (trim_high + start_dimension)
        }
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

        BitVec::from_bytes(&buf[..]).iter()
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


#[cfg(test)] 
mod test {
    use db::{Value, Window, SubstitutionVariant, DeletionVariant};

    // Vec<u8> tests
    /* I don't think this is a valid test...
    #[test]
    fn test_window_min_start_and_finish_vec_u8() {
        let a = vec![1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8];
        let b = vec![];

        assert_eq!(a.window(0,0), b);
    }
    */

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
            (vec![255u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8], 0u32),
            (vec![0u8, 255u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8], 1u32),
            (vec![0u8, 0u8, 255u8, 0u8, 0u8, 0u8, 0u8, 0u8], 2u32),
            (vec![0u8, 0u8, 0u8, 255u8, 0u8, 0u8, 0u8, 0u8], 3u32),
        ];

        assert_eq!(a.deletion_variants(4).collect::<Vec<(Vec<u8>, u32)>>(), expected);
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
    /* I don't think this is a valid test...
    #[test]
    #[should_panic]
    fn test_window_min_start_and_finish_u8() {
        let a = 0b10000001u8;
        let b = 0b00000000u8;

        assert_eq!(a.window(0,0), b);
    }
    */

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

        assert_eq!(a.substitution_variants(4).collect::<Vec<u8>>(), expected);
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

        assert_eq!(a.deletion_variants(4).collect::<Vec<(u8, u8)>>(), expected);
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

        assert_eq!(a.substitution_variants(4).collect::<Vec<usize>>(), expected);
    }

    #[test]
    fn test_deletion_variants_usize() {
        let a = 0b00000000usize;
        let expected = vec![
            (0b00000001usize, 0u32),
            (0b00000010usize, 1u32),
            (0b00000100usize, 2u32),
            (0b00001000usize, 3u32),
        ];

        assert_eq!(a.deletion_variants(4).collect::<Vec<(usize, u32)>>(), expected);
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
}
