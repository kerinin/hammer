// use std::u8;

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
        let trim_high = 8 - (start_dimension + dimensions);

        if trim_high >= 8 {
            0u8
        } else {
            (self << trim_high) >> (trim_high + start_dimension)
        }
    }
}

impl Window<u64> for u64 {
    fn window(&self, start_dimension: usize, dimensions: usize) -> u64 {
        let trim_high = 64 - (start_dimension + dimensions);

        if trim_high >= 64 {
            0
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

impl Window<(u8, u8)> for (u8, u8) {
    fn window(&self, start_dimension: usize, dimensions: usize) -> (u8, u8) {
        let &(value, delete_bit) = self;

        //  2/4        11111111
        //              ^<--^
        //  << 1       11111110
        //  >> 1+2     00011111
        let trim_high = 8 - (start_dimension + dimensions);

        let mut new_delete_bit = delete_bit.clone();
        if start_dimension < (delete_bit as usize) && (start_dimension + dimensions) <= (delete_bit as usize) {
            new_delete_bit = 0;
        }

        if trim_high >= 8 {
            (0u8, new_delete_bit)
        } else {
            ((value << trim_high) >> (trim_high + start_dimension), new_delete_bit)
        }
    }
}

impl Window<(u64, u8)> for (u64, u8) {
    fn window(&self, start_dimension: usize, dimensions: usize) -> (u64, u8) {
        let &(value, delete_bit) = self;

        let trim_high = 64 - (start_dimension + dimensions);

        let mut new_delete_bit = delete_bit.clone();
        if start_dimension < (delete_bit as usize) && (start_dimension + dimensions) <= (delete_bit as usize) {
            new_delete_bit = 0;
        }

        if trim_high >= 64 {
            (0, new_delete_bit)
        } else {
            ((value << trim_high) >> (trim_high + start_dimension), new_delete_bit)
        }
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

    // u64 tests

    #[test]
    fn test_window_min_start_and_finish_u64() {
        let a = 0b10000001u64;
        let b = 0b00000001u64;

        assert_eq!(a.window(0,1), b);
    }

    #[test]
    fn test_window_max_start_u64() {
        let a = 0b10000001u64;
        let b = 0b00000001u64;

        assert_eq!(a.window(7,1), b);
    }

    #[test]
    fn test_window_min_start_and_max_finish_u64() {
        let a = 0b10000001u64;
        let b = 0b10000001u64;

        assert_eq!(a.window(0,8), b);
    }

    #[test]
    fn test_window_n_start_and_max_finish_u64() {
        let a = 0b11000011u64;
        let b = 0b01100001u64;

        assert_eq!(a.window(1,7), b);
    }

    #[test]
    fn test_window_min_start_and_n_finish_u64() {
        let a = 0b11000011u64;
        let b = 0b01000011u64;

        assert_eq!(a.window(0,7), b);
    }

    #[test]
    fn test_window_n_start_and_n_finish_u64() {
        let a = 0b11111000u64;
        let b = 0b00000011u64;

        assert_eq!(a.window(3,2), b);
    }
}
