use std::iter;
use std::ops;
use std::fmt;

use std::collections::BitVec;

use db::value::{Value, Window, SubstitutionVariant, Hamming};

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub struct BitMatrix {
    data: Vec<Vec<u8>>,
}

// bitmatrix![[1,2,3], [4,5,6]]
#[macro_export]
macro_rules! bitmatrix {
    [ $( [ $( $x:expr ),* ] ),* ] => { BitMatrix::new(vec![ $( vec![ $( $x ),* ] )* ]) };
}

pub trait AsBitMatrix {
    fn as_bitmatrix(self) -> BitMatrix;
    fn from_bitmatrix(BitMatrix) -> Self;
}

impl BitMatrix {
    pub fn new(data: Vec<Vec<u8>>) -> BitMatrix {
        BitMatrix { data: data }
    }

    pub fn rows(&self) -> usize {
        self.data.len()
    }
    pub fn columns(&self) -> usize {
        self.data[0].len() * 8
    }

    // SUPER inefficient, intended as a placeholder
    pub fn transpose(&self) -> Self {
        let source_x = self.data.len();
        let source_y = self.data[0].len();
        let mut out = vec![vec![0; source_x]; source_y];

        for from_x in 0..source_x {
            for from_y in 0..source_y {
                out[from_y][from_x] = self.data[from_x][from_y];
            }
        }

        return BitMatrix {data: out};
    }

    fn permute(&self, dimension: usize) -> Vec<BitMatrix> {
        let byte_offset = dimension / 8;
        let bit_offset = dimension % 8;
        let toggle = 1u8 << bit_offset;

        range(0, self.data.len()).map(|i| {
            let mut permuted = self.clone();
            permuted.data[i][byte_offset] = permuted.data[i][byte_offset] ^ toggle;
            permuted
        }).collect::<Vec<BitMatrix>>()
    }
}

impl Value for BitMatrix {}
impl Window for BitMatrix {
    fn window(&self, start_dimension: usize, dimensions: usize) -> BitMatrix {
        let trim_high = self.columns() - (start_dimension + dimensions);

        (self.clone() << trim_high) >> (trim_high + start_dimension)
    }
}
impl SubstitutionVariant for BitMatrix {
    fn substitution_variants(&self, dimensions: usize) -> Vec<BitMatrix> {
        return range(0, dimensions)
            .flat_map(|i| self.permute(i).into_iter() )
            .collect::<Vec<BitMatrix>>();
    }
}
impl Hamming for BitMatrix {
    fn hamming(&self, other: &BitMatrix) -> usize {
        let all = BitVec::from_elem(self.columns(), true);

        let shared_dimensions = self.data.iter()
            .zip(other.data.iter())
            .fold(all, |mut memo, (self_i, other_i)| {
                let xor_bytes_i = self_i.iter().zip(other_i.iter()).map(|(self_byte, other_byte)| {
                    self_byte ^ other_byte
                }).collect::<Vec<u8>>();

                memo.intersect(&BitVec::from_bytes(xor_bytes_i.as_slice()));
                memo
            });

        shared_dimensions.iter().filter(|x| *x).count()
    }
}

impl ops::BitXor for BitMatrix {
    type Output = BitMatrix;

    /*
     * Returns the result of bitxor-ing each byte of self and other.
     * If other is shorter than self, 0 will be used, if self is shorter than
     * other, the trailing bytes of other will be ignored
     */
    fn bitxor(self, other: Self) -> Self {
        let data = range(0, self.data.len()).map(|i| {
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
        let data = range(0, self.data.len()).map(|i| {
            self.data[i].iter()
                .zip(other.data[i].iter())
                .map(|(self_byte, other_byte)| self_byte.clone() & *other_byte ) // bitand
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

impl fmt::Binary for BitMatrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "[");
        for outer in self.data.iter() {
            write!(f, "[");
            for inner in outer.iter() {
                write!(f, "{:08b}", inner);
            }
            write!(f, "]");
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod test {
    use db::bit_matrix::BitMatrix;


    /*
     * Need to test:
     * =============
     * AsBitMatrix impls (none actually exist atm)
     * Macro
     * transposition
     * Benchmark
     *
     */

    #[test]
    fn bitxor_equally_sized_vectors() {
        let a = bitmatrix![[0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = bitmatrix![[0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]];
        let c = bitmatrix![[0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8]];

        assert_eq!(a ^ b, c);
    }

    #[test]
    fn bitxor_left_vector_longer() {
        let a = bitmatrix![[0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = bitmatrix![[0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]];
        let c = bitmatrix![[0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];

        assert_eq!(a ^ b, c);
    }

    #[test]
    fn bitxor_right_vector_longer() {
        let a = bitmatrix![[0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = bitmatrix![[0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let c = bitmatrix![[0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8]];

        assert_eq!(a ^ b, c);
    }

    #[test]
    fn bitand_equally_sized_vectors() {
        let a = bitmatrix![[0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = bitmatrix![[0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]];
        let c = bitmatrix![[0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8]];

        assert_eq!(a & b, c);
    }

    #[test]
    fn bitand_left_vector_longer() {
        let a = bitmatrix![[0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = bitmatrix![[0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]];
        let c = bitmatrix![[0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8]];

        assert_eq!(a & b, c);
    }

    #[test]
    fn bitand_right_vector_longer() {
        let a = bitmatrix![[0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = bitmatrix![[0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let c = bitmatrix![[0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8]];

        assert_eq!(a & b, c);
    }

    #[test]
    fn shl_zero() {
        let a = bitmatrix![[0b00000000u8, 0b11111111u8]];
        let b = bitmatrix![[0b00000000u8, 0b11111111u8]];

        assert_eq!(a << 0, b);
    }

    #[test]
    fn shl_less_than_vector_length() {
        let a = bitmatrix![[0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = bitmatrix![[0b00001111u8, 0b11110000u8, 0b00000000u8]];

        assert_eq!(a << 12, b);
    }

    #[test]
    fn shl_vector_length() {
        let a = bitmatrix![[0b00000000u8]];
        let b = bitmatrix![[]];

        assert_eq!(a << 8, b);
    }

    #[test]
    fn shl_more_than_vector_length() {
        let a = bitmatrix![[0b00000000u8]];
        let b = bitmatrix![[]];

        assert_eq!(a << 12, b);
    }

    #[test]
    fn shr_zero() {
        let a = bitmatrix![[0b00000000u8, 0b11111111u8]];
        let b = bitmatrix![[0b00000000u8, 0b11111111u8]];

        assert_eq!(a >> 0, b);
    }

    #[test]
    fn shr_less_than_vector_length() {
        let a = bitmatrix![[0b00000000u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]];
        let b = bitmatrix![[0b00000000u8, 0b00001111u8, 0b11110000u8]];

        assert_eq!(a >> 12, b);
    }

    #[test]
    fn shr_vector_length() {
        let a = bitmatrix![[0b00000000u8]];
        let b = bitmatrix![[]];

        assert_eq!(a >> 8, b);
    }

    #[test]
    fn shr_more_than_vector_length() {
        let a = bitmatrix![[0b00000000u8]];
        let b = bitmatrix![[]];

        assert_eq!(a >> 12, b);
    }
}

