use std::iter;
use std::ops;

use bit_matrix::BitMatrix;

impl ops::BitXor for BitMatrix {
    type Output = BitMatrix;

    /*
     * Returns the result of bitxor-ing each byte of self and other.
     * If other is shorter than self, 0 will be used, if self is shorter than
     * other, the trailing bytes of other will be ignored
     */
    fn bitxor(self, other: Self) -> Self {
        let data = (0..self.data.len()).map(|i| {
            let zero: &u8 = &0;
            let other_then_zero = other.data[i].iter().chain(iter::repeat(zero));

            self.data[i].iter()
                .zip(other_then_zero)
                .map(|(self_byte, other_byte)| self_byte.clone() ^ *other_byte ) // bitxor
                .collect::<Vec<u8>>()
        }).collect::<Vec<Vec<u8>>>();

        return BitMatrix {data: data};
    }
}

impl ops::BitAnd for BitMatrix {
    type Output = BitMatrix;

    /*
     * Returns the result of bitand-ing each byte of self and other.
     * If other is shorter than self, self will be truncated to the same length.
     * If self is shorter than other, the trailing bytes of other will be ignored.
     */
    fn bitand(self, other: Self) -> Self {
        let data = (0..self.data.len()).map(|i| {
            self.data[i].iter()
                .zip(other.data[i].iter())
                .map(|(self_byte, other_byte)| self_byte.clone() & *other_byte ) // bitand
                .collect::<Vec<u8>>()
        }).collect::<Vec<Vec<u8>>>();

        return BitMatrix {data: data};
    }
}

impl ops::BitOr for BitMatrix {
    type Output = BitMatrix;

    /*
     * Returns the result of bitand-ing each byte of self and other.
     * If other is shorter than self, self will be truncated to the same length.
     * If self is shorter than other, the trailing bytes of other will be ignored.
     */
    fn bitor(self, other: Self) -> Self {
        let data = (0..self.data.len()).map(|i| {
            self.data[i].iter()
                .zip(other.data[i].iter())
                .map(|(self_byte, other_byte)| self_byte.clone() | *other_byte ) // bitand
                .collect::<Vec<u8>>()
        }).collect::<Vec<Vec<u8>>>();

        return BitMatrix {data: data};
    }
}

impl ops::Shl<usize> for BitMatrix {
    type Output = BitMatrix;

    /*
     * Returns a new byte array with RHS bits removed from the left side, and
     * pads the left-most byte with zeros (if necessary)
     */
    fn shl(self, rhs: usize) -> Self {
        let data = self.data.iter().map(|outer| {
            if rhs == 0 { 
                outer.clone()
            } else {
                let to_drop = rhs / 8;
                let to_shift = rhs % 8;
                let to_unshift = 8 - to_shift;
                let mut out: Vec<u8> = vec![];

                // Drop elements containing only bits to be shifted off
                let mut iter = outer.iter().skip(to_drop).peekable();

                // If we don't need to shift any bits, we're done
                while to_shift > 0 {
                    match iter.next() {

                        Some(&this_byte) => {
                            // Shift some bits 
                            let out_byte = this_byte << to_shift; // shl

                            match iter.peek() {

                                // There's another byte - shift some of its bits 
                                // into this byte
                                Some(&next_byte) => {
                                    let bits_from_next_byte = next_byte >> to_unshift; // shr

                                    out.push(out_byte ^ bits_from_next_byte); //bitxor
                                },

                                // We're on the last element, so we don't need to pull
                                // bits from the next one
                                None => {
                                    out.push(out_byte);
                                },
                            }
                        },

                        // All done!
                        None => break,
                    }
                }

                out
            }
        }).collect::<Vec<Vec<u8>>>();

        return BitMatrix {data: data};
    }
}

impl ops::Shr<usize> for BitMatrix {
    type Output = BitMatrix;

    /*
     * Returns a new byte array with RHS bits removed from the left side, and
     * pads the right-most byte with zeros (if necessary)
     */
    fn shr(self, rhs: usize) -> Self {
        let data = self.data.iter().map(|outer| {
            if rhs == 0 { 
                outer.clone()
            } else {
                let to_drop = rhs / 8;
                let to_shift = rhs % 8;
                let to_unshift = 8 - to_shift;
                let mut out: Vec<u8> = vec![];

                // Drop elements containing only bits to be shifted off
                let mut iter = outer.iter().rev().skip(to_drop).peekable();

                // If we don't need to shift any bits, we're done
                while to_shift > 0 {
                    match iter.next() {

                        Some(&this_byte) => {
                            // Shift some bits 
                            let out_byte = this_byte >> to_shift; // shr

                            match iter.peek() {

                                // There's another byte - shift some of its bits 
                                // into this byte
                                Some(&next_byte) => {
                                    let bits_from_next_byte = next_byte << to_unshift; // shl

                                    out.insert(0, out_byte ^ bits_from_next_byte);
                                },

                                // We're on the last element, so we don't need to pull
                                // bits from the next one
                                None => {
                                    out.insert(0, out_byte);
                                },
                            }
                        },

                        // All done!
                        None => break,
                    }
                }

                out
            }
        }).collect::<Vec<Vec<u8>>>();

        return BitMatrix {data: data};
    }
}

impl ops::Not for BitMatrix {
    type Output = BitMatrix;

    fn not(self) -> BitMatrix {
        let data = (0..self.data.len()).map(|i| {
            self.data[i].iter()
            .map(|byte| !byte)
            .collect::<Vec<u8>>()
        }).collect::<Vec<Vec<u8>>>();

        return BitMatrix {data: data};
    }
}


#[cfg(test)]
mod test {
    extern crate quickcheck;
    use self::quickcheck::quickcheck;

    use bit_matrix::{BitMatrix, AsBitMatrix};

    #[test]
    fn boolean_algebra_and() {
        fn prop(a: u64, b: u64) -> quickcheck::TestResult {
            let a_as_bm = a.as_bit_matrix();
            let b_as_bm = b.as_bit_matrix();

            let left = a_as_bm.clone() & b_as_bm.clone();
            let right = !((!a_as_bm.clone()) | (!b_as_bm.clone()));
                
            quickcheck::TestResult::from_bool(left == right)
        }
        quickcheck(prop as fn(u64, u64) -> quickcheck::TestResult);
    }

    #[test]
    fn boolean_algebra_or() {
        fn prop(a: u64, b: u64) -> quickcheck::TestResult {
            let a_as_bm = a.as_bit_matrix();
            let b_as_bm = b.as_bit_matrix();

            let left = a_as_bm.clone() | b_as_bm.clone();
            let right = !((!a_as_bm.clone()) & (!b_as_bm.clone()));
                
            quickcheck::TestResult::from_bool(left == right)
        }
        quickcheck(prop as fn(u64, u64) -> quickcheck::TestResult);
    }

    #[test]
    fn xor_algebra() {
        fn prop(a: u64, b: u64) -> quickcheck::TestResult {
            let a_as_bm = a.as_bit_matrix();
            let b_as_bm = b.as_bit_matrix();

            let left = a_as_bm.clone() ^ b_as_bm.clone();
            let right = (a_as_bm.clone() | b_as_bm.clone()) & !(a_as_bm.clone() & b_as_bm.clone());
                
            quickcheck::TestResult::from_bool(left == right)
        }
        quickcheck(prop as fn(u64, u64) -> quickcheck::TestResult);
    }

    #[test]
    #[should_panic]
    fn shl_associative() {
        fn prop(a: u64, b: u8) -> quickcheck::TestResult {
            let shift = (b >> 2) as usize; // limit to range 0-64
                
            quickcheck::TestResult::from_bool(
                (a.as_bit_matrix() << shift) == (a << shift).as_bit_matrix()
                )
        }
        quickcheck(prop as fn(u64, u8) -> quickcheck::TestResult);
    }

    #[test]
    #[should_panic]
    fn shr_associative() {
        fn prop(a: u64, b: u8) -> quickcheck::TestResult {
            let shift = (b >> 2) as usize; // limit to range 0-64
                
            quickcheck::TestResult::from_bool(
                (a.as_bit_matrix() >> shift) == (a >> shift).as_bit_matrix()
                )
        }
        quickcheck(prop as fn(u64, u8) -> quickcheck::TestResult);
    }

    #[test]
    fn bitxor_equally_sized_vectors() {
        let a = BitMatrix::new(vec![vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]]);
        let c = BitMatrix::new(vec![vec![0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8]]);

        assert_eq!(a ^ b, c);
    }

    #[test]
    fn bitxor_left_vector_longer() {
        let a = BitMatrix::new(vec![vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]]);
        let c = BitMatrix::new(vec![vec![0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]]);

        assert_eq!(a ^ b, c);
    }

    #[test]
    fn bitxor_right_vector_longer() {
        let a = BitMatrix::new(vec![vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]]);
        let c = BitMatrix::new(vec![vec![0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8]]);

        assert_eq!(a ^ b, c);
    }

    #[test]
    fn bitand_equally_sized_vectors() {
        let a = BitMatrix::new(vec![vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]]);
        let c = BitMatrix::new(vec![vec![0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8]]);

        assert_eq!(a & b, c);
    }

    #[test]
    fn bitand_left_vector_longer() {
        let a = BitMatrix::new(vec![vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]]);
        let c = BitMatrix::new(vec![vec![0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8]]);

        assert_eq!(a & b, c);
    }

    #[test]
    fn bitand_right_vector_longer() {
        let a = BitMatrix::new(vec![vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]]);
        let c = BitMatrix::new(vec![vec![0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8]]);

        assert_eq!(a & b, c);
    }

    #[test]
    fn shl_zero() {
        let a = BitMatrix::new(vec![vec![0b00000000u8, 0b11111111u8]]);
        let b = BitMatrix::new(vec![vec![0b00000000u8, 0b11111111u8]]);

        assert_eq!(a << 0, b);
    }

    #[test]
    fn shl_less_than_vector_length() {
        let a = BitMatrix::new(vec![vec![0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![0b00001111u8, 0b11110000u8, 0b00000000u8]]);

        assert_eq!(a << 12, b);
    }

    #[test]
    fn shl_vector_length() {
        let a = BitMatrix::new(vec![vec![0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![]]);

        assert_eq!(a << 8, b);
    }

    #[test]
    fn shl_more_than_vector_length() {
        let a = BitMatrix::new(vec![vec![0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![]]);

        assert_eq!(a << 12, b);
    }

    #[test]
    fn shr_zero() {
        let a = BitMatrix::new(vec![vec![0b00000000u8, 0b11111111u8]]);
        let b = BitMatrix::new(vec![vec![0b00000000u8, 0b11111111u8]]);

        assert_eq!(a >> 0, b);
    }

    #[test]
    fn shr_less_than_vector_length() {
        let a = BitMatrix::new(vec![vec![0b00000000u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![0b00000000u8, 0b00001111u8, 0b11110000u8]]);

        assert_eq!(a >> 12, b);
    }

    #[test]
    fn shr_vector_length() {
        let a = BitMatrix::new(vec![vec![0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![]]);

        assert_eq!(a >> 8, b);
    }

    #[test]
    fn shr_more_than_vector_length() {
        let a = BitMatrix::new(vec![vec![0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![]]);

        assert_eq!(a >> 12, b);
    }

}
