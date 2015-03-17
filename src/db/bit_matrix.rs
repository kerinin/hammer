use std::iter;
use std::ops;
use std::fmt;

use std::collections::BitVec;

use db::value::{Value, Window, SubstitutionVariant, DeletionVariant, Hamming};

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

    fn mask(&self, dimension: usize) -> BitMatrix {
        let byte_offset = self.data[0].len() - (dimension / 8) - 1;
        let bit_offset = dimension % 8;
        let toggle = 1u8 << bit_offset;
        let mut masked = self.clone();

        for i in 0..self.data.len() {
            masked.data[i][byte_offset] = masked.data[i][byte_offset] ^ toggle;
        }

        masked
    }

    fn permute(&self, dimension: usize) -> Vec<BitMatrix> {
        let byte_offset = self.data[0].len() - (dimension / 8) - 1;
        let bit_offset = dimension % 8;
        let toggle = 1u8 << bit_offset;

        (0..self.data.len()).map(|i| {
            let mut permuted = self.clone();
            permuted.data[i][byte_offset] = permuted.data[i][byte_offset] ^ toggle;
            permuted
        }).collect::<Vec<BitMatrix>>()
    }
}

impl Value for BitMatrix {}
impl Value for (BitMatrix, usize) {}

impl Window for BitMatrix {
    fn window(&self, start_dimension: usize, dimensions: usize) -> BitMatrix {
        let trim_high = self.columns() - (start_dimension + dimensions);

        (self.clone() << trim_high) >> (trim_high + start_dimension)
    }
}
impl SubstitutionVariant for BitMatrix {
    fn substitution_variants(&self, dimensions: usize) -> Vec<BitMatrix> {
        return (0..dimensions)
            .flat_map(|i| self.permute(i).into_iter() )
            .collect::<Vec<BitMatrix>>();
    }
}
impl DeletionVariant for BitMatrix {
    type Output = (BitMatrix, usize);

    fn deletion_variants(&self, dimensions: usize) -> Vec<(BitMatrix, usize)> {
        return (0..dimensions)
            .map(|i| (self.mask(i), i))
            .collect::<Vec<(BitMatrix, usize)>>();
    }
}
impl Hamming for BitMatrix {
    /*
     * Column-major hamming distance operates like an AND of the column-wise XOR's
     */
    fn hamming(&self, other: &BitMatrix) -> usize {
        let all = BitVec::from_elem(self.columns(), true);

        let shared_dimensions = self.data.iter()
            .zip(other.data.iter())
            .fold(all, |mut memo, (self_i, other_i)| {
                let xor_bytes_i = self_i.iter().zip(other_i.iter()).map(|(self_byte, other_byte)| {
                    // 1: value is shared
                    // 0: value is different
                    !(self_byte ^ other_byte)
                }).collect::<Vec<u8>>();

                // Find values which were shared previously and are shared currently
                memo.intersect(&BitVec::from_bytes(xor_bytes_i.as_slice()));
                memo
            });

        // Return the count of values which were not shared between all rows
        shared_dimensions.iter().filter(|x| !*x).count()
    }
    
    // NOTE: This can be expedited by bailing early...
    fn hamming_lte(&self, other: &BitMatrix, bound: usize) -> bool {
        self.hamming(other) <= bound
    }
}
impl Hamming for (BitMatrix, usize) {
    fn hamming(&self, other: &(BitMatrix, usize)) -> usize {
        let (self_value, self_deleted_index) = self.clone();
        let (other_value, other_deleted_index) = other.clone();

        let mut shared_dimensions = BitVec::from_elem(self_value.columns(), true);

        // Faster than building the bitmap and XOR-ing...
        if (self_deleted_index != other_deleted_index) {
            shared_dimensions.set(self_value.columns() - self_deleted_index - 1, false);
            shared_dimensions.set(other_value.columns() - other_deleted_index - 1, false);
        }

        for i in 0..self_value.rows() {
            let xor_bytes_i = self_value.data[i].iter().zip(other_value.data[i].iter()).map(|(self_byte, other_byte)| {
                !(self_byte ^ other_byte)
            }).collect::<Vec<u8>>();

            shared_dimensions.intersect(&BitVec::from_bytes(xor_bytes_i.as_slice()));
        }

        shared_dimensions.iter().filter(|x| !*x).count()
    }
    
    fn hamming_lte(&self, other: &(BitMatrix, usize), bound: usize) -> bool {
        let (self_value, self_deleted_index) = self.clone();
        let (other_value, other_deleted_index) = other.clone();

        let mut shared_dimensions = BitVec::from_elem(self_value.columns(), true);

        // Faster than building the bitmap and XOR-ing...
        if (self_deleted_index != other_deleted_index) {
            shared_dimensions.set(self_value.columns() - self_deleted_index - 1, false);
            shared_dimensions.set(other_value.columns() - other_deleted_index - 1, false);
        }

        for i in 0..self_value.rows() {
            let xor_bytes_i = self_value.data[i].iter().zip(other_value.data[i].iter()).map(|(self_byte, other_byte)| {
                !(self_byte ^ other_byte)
            }).collect::<Vec<u8>>();

            shared_dimensions.intersect(&BitVec::from_bytes(xor_bytes_i.as_slice()));
            if shared_dimensions.iter().filter(|x| !*x).count() > bound {
                return true;
            }
        }

        shared_dimensions.iter().filter(|x| !*x).count() > bound
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
    extern crate quickcheck;
    use self::quickcheck::quickcheck;

    use db::bit_matrix::BitMatrix;
    use db::value::{SubstitutionVariant, DeletionVariant, Hamming};

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

    #[test]
    fn substitution_variants() {
        let a = bitmatrix![[0b00000000u8, 0b00000000u8]];
        let expected = vec![
            bitmatrix![[0b00000000u8, 0b00000001u8]],
            bitmatrix![[0b00000000u8, 0b00000010u8]],
            bitmatrix![[0b00000000u8, 0b00000100u8]],
            bitmatrix![[0b00000000u8, 0b00001000u8]],
            bitmatrix![[0b00000000u8, 0b00010000u8]],
            bitmatrix![[0b00000000u8, 0b00100000u8]],
            bitmatrix![[0b00000000u8, 0b01000000u8]],
            bitmatrix![[0b00000000u8, 0b10000000u8]],
            bitmatrix![[0b00000001u8, 0b00000000u8]],
            bitmatrix![[0b00000010u8, 0b00000000u8]],
        ];

        assert_eq!(a.substitution_variants(10), expected);
    }

    #[test]
    fn deletion_variants() {
        let a = bitmatrix![[0b00000000u8, 0b00000000u8]];
        let expected = vec![
            (bitmatrix![[0b00000000u8, 0b00000001u8]], 0usize),
            (bitmatrix![[0b00000000u8, 0b00000010u8]], 1usize),
            (bitmatrix![[0b00000000u8, 0b00000100u8]], 2usize),
            (bitmatrix![[0b00000000u8, 0b00001000u8]], 3usize),
            (bitmatrix![[0b00000000u8, 0b00010000u8]], 4usize),
            (bitmatrix![[0b00000000u8, 0b00100000u8]], 5usize),
            (bitmatrix![[0b00000000u8, 0b01000000u8]], 6usize),
            (bitmatrix![[0b00000000u8, 0b10000000u8]], 7usize),
            (bitmatrix![[0b00000001u8, 0b00000000u8]], 8usize),
            (bitmatrix![[0b00000010u8, 0b00000000u8]], 9usize),
        ];

        assert_eq!(a.deletion_variants(10), expected);
    }

    #[test]
    fn deletion_hamming_equal_usize_u8() {
        let a = (bitmatrix![[0b11111111u8]], 0usize);
        let b = (bitmatrix![[0b11111111u8]], 0usize);

        let c = (bitmatrix![[0b00000001u8]], 0usize);
        let d = (bitmatrix![[0b00000001u8]], 0usize);

        assert_eq!(a.hamming(&b), 0);
        assert_eq!(c.hamming(&d), 0);
    }

    #[test]
    fn deletion_hamming_binary_unequal_usize_u8() {
        let a = (bitmatrix![[0b11111111u8]], 0usize);
        let b = (bitmatrix![[0b01111111u8]], 0usize);

        let c = (bitmatrix![[0b10000001u8]], 0usize);
        let d = (bitmatrix![[0b00000001u8]], 0usize);

        assert_eq!(a.hamming(&b), 1);
        assert_eq!(c.hamming(&d), 1);
    }

    #[test]
    fn deletion_hamming_deleted_unequal_usize_u8() {
        let a = (bitmatrix![[0b11111111u8]], 0usize);
        let b = (bitmatrix![[0b11111111u8]], 1usize);

        let c = (bitmatrix![[0b00000001u8]], 0usize);
        let d = (bitmatrix![[0b00000010u8]], 1usize);

        assert_eq!(a.hamming(&b), 2);
        assert_eq!(c.hamming(&d), 2);
    }

    #[test]
    fn deletion_hamming_binary_and_deleted_unequal_usize_u8() {
        let a = (bitmatrix![[0b11111111u8]], 0usize);
        let b = (bitmatrix![[0b01111111u8]], 1usize);

        let c = (bitmatrix![[0b10000001u8]], 0usize);
        let d = (bitmatrix![[0b00000010u8]], 1usize);

        assert_eq!(a.hamming(&b), 3);
        assert_eq!(c.hamming(&d), 3);
    }
}

