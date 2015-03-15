use std;
use std::hash;
use std::cmp;
use std::clone;
use std::ops;
use std::fmt;
use std::num::Int;

//pub trait Value: Permutable + cmp::Eq + hash::Hash + clone::Clone + fmt::Show + iter::FromIterator<u8> {
pub trait Value: hash::Hash + cmp::Eq + clone::Clone + ops::BitXor + ops::BitAnd + ops::Shl<usize> + fmt::Debug + fmt::Binary {
    fn window(&self, start_dimension: usize, dimensions: usize) -> Self;
    fn permutations(&self, n: usize) -> Vec<Self>;
    fn hamming(&self, rhs: &Self) -> usize;
}

impl Value for u8 {
    /* 
     * `start_dimension` the index of the 1st dimension to include in the slice, 
     *      0-indexed from least significant
     * `dimensions` the total number of dimensions to include
     */
    fn window(&self, start_dimension: usize, dimensions: usize) -> u8 {
        //  2/4        11111111
        //              ^<--^
        //  << 1       11111110
        //  >> 1+2     00011111

        let bits = std::u8::BITS as usize;
        let trim_high = bits - (start_dimension + dimensions);

        (self << trim_high) >> (trim_high + start_dimension)
    }

    fn permutations(&self, dimensions: usize) -> Vec<u8> {
        let bits = std::u8::BITS as usize;
        return range(0, dimensions)
            .map(|i| {
                let delta = 1u8 << i;
                self.clone() ^ delta
            })
            .collect::<Vec<u8>>();
    }

    fn hamming(&self, other: &u8) -> usize {
        (*self ^ *other).count_ones() as usize // bitxor
    }
}

impl Value for usize {
    fn window(&self, start_dimension: usize, dimensions: usize) -> usize {
        let bits = std::usize::BITS as usize;
        let trim_high = bits - (start_dimension + dimensions);

        (self << trim_high) >> (trim_high + start_dimension)
    }

    fn permutations(&self, dimensions: usize) -> Vec<usize> {
        let bits = std::usize::BITS as usize;
        return range(0, dimensions)
            .map(|i| {
                let delta = 1usize << i;
                self.clone() ^ delta
            })
            .collect::<Vec<usize>>();
    }

    fn hamming(&self, other: &usize) -> usize {
        (*self ^ *other).count_ones() as usize // bitxor
    }
}

#[cfg(test)]
mod test {
    use db::value::Value;

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
    fn test_permutation_u8() {
        let a = 0b00000000u8;
        let expected = vec![
            0b00000001u8,
            0b00000010u8,
            0b00000100u8,
            0b00001000u8,
        ];

        assert_eq!(a.permutations(4), expected);
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
    fn test_permutation_usize() {
        let a = 0b00000000usize;
        let expected = vec![
            0b00000001usize,
            0b00000010usize,
            0b00000100usize,
            0b00001000usize,
        ];

        assert_eq!(a.permutations(4), expected);
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
