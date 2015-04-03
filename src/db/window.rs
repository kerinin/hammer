use std;

pub trait Window<T> {
    /// Subsample on a set of dimensions
    ///
    /// `start_dimension` the index of the 1st dimension to include in the slice, 
    ///      0-indexed from least significant
    /// `dimensions` the total number of dimensions to include
    ///
    fn window(&self, start_dimension: usize, dimensions: usize) -> T;
}

impl Window<u8> for u8 {
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

impl Window<usize> for usize {
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

impl Window<Vec<u8>> for Vec<u8> {
    fn window(&self, start_dimension: usize, dimensions: usize) -> Vec<u8> {
        self[start_dimension..(start_dimension + dimensions)].to_vec()
    }
}

#[cfg(test)] 
mod test {
    use db::window::*;

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
}
