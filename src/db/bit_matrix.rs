use std::iter;
use std::ops;
use std::usize;
use std::mem;
use std::vec;
use std::u8;

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub struct BitMatrix {
    data: Vec<Vec<u8>>,
}

pub trait AsBitMatrix {
    fn as_bitmatrix(self) -> BitMatrix;
    fn from_bitmatrix(BitMatrix) -> Self;
}

impl AsBitMatrix for usize {
    fn as_bitmatrix(self) -> BitMatrix {
        let bytes: [u8; usize::BYTES as usize] = unsafe { mem::transmute(self) };
        let vector = unsafe { vec::Vec::from_raw_buf(&bytes[0], bytes.len()) };

        BitMatrix {data: vec![vector]}
    }

    fn from_bitmatrix(bm: BitMatrix) -> usize {
        return bm.data[0].iter().fold(0, |memo, i| {
            // Possibly should be SHL, or other variations...
            memo << 8;
            memo & (*i as usize)
        })
    }
}

impl BitMatrix {
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

    pub fn transform(&self, shift: usize, mask: usize) -> BitMatrix {
        let shifted: BitMatrix = self.clone() << shift;

        let full_byte_count = mask / 8;
        let tail_dimensions = mask % 8;
        let partial_mask = 0b11111111u8 << (8-tail_dimensions);

        let mut mask = iter::repeat(0b11111111u8).take(full_byte_count).collect::<Vec<u8>>();
        mask.push(partial_mask);

        let transformed = shifted.data.iter().map(|outer| {
            range(0, mask.len()).map(|i| outer[i] & mask[i]).collect::<Vec<u8>>()
        }).collect::<Vec<Vec<u8>>>();

        return BitMatrix {data: transformed};
    }

    pub fn permutations(&self, n: usize) -> Vec<BitMatrix> {
        return vec![self.clone()];
        /*
        // NOTE: Probably needs a full rewrite...
        let bv = BitVec::from_bytes(self.data.as_slice());

        return range(0usize, n)
            .map(|i| -> Vec<u8> {
                let mut permutation = bv.clone();
                match permutation.get(i) {
                    Some(old_val) => permutation.set(i, !old_val),
                    // NOTE: If more permutations were requested than can be generated, 
                    // we'll just pad the end with unmodified versions
                    _ => () 
                }

                permutation.to_bytes()
            })
        .collect::<Vec<Vec<u8>>>();
        */
    }

    pub fn hamming(&self, other: &BitMatrix) -> usize {
        // Might want to just use bit vectors here...
        let all = vec![u8::MAX; self.data[0].len()];

        let distance = range(0, self.data.len()).fold(all, |mut shared, i| {
            for (self_i, other_i) in self.data[i].iter().zip(other.data[i].iter()) {
                shared[i] = shared[i] & (self_i ^ other_i);
            };
            shared
        });

        return 0;
    }

    pub fn within_hamming(&self, bound: usize, other: &BitMatrix) -> bool {
        // NOTE: same as hamming, except exit early if possible
        false
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

#[cfg(test)]
mod test {
    use db::permutable::Permutable;

    #[test]
    fn bitxor_equally_sized_vectors() {
        let a = [[0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = [[0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]];
        let c = [[0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8]];

        assert_eq!(a ^ b, c);
    }

    #[test]
    fn bitxor_left_vector_longer() {
        let a = [[0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = [[0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]];
        let c = [[0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];

        assert_eq!(a ^ b, c);
    }

    #[test]
    fn bitxor_right_vector_longer() {
        let a = [[0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = [[0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let c = [[0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8]];

        assert_eq!(a ^ b, c);
    }

    #[test]
    fn bitand_equally_sized_vectors() {
        let a = [[0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = [[0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]];
        let c = [[0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8]];

        assert_eq!(a & b, c);
    }

    #[test]
    fn bitand_left_vector_longer() {
        let a = [[0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = [[0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]];
        let c = [[0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8]];

        assert_eq!(a & b, c);
    }

    #[test]
    fn bitand_right_vector_longer() {
        let a = [[0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = [[0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let c = [[0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8]];

        assert_eq!(a & b, c);
    }

    #[test]
    fn shl_zero() {
        let a = [[0b00000000u8, 0b11111111u8]];
        let b = [[0b00000000u8, 0b11111111u8]];

        assert_eq!(a << 0, b);
    }

    #[test]
    fn shl_less_than_vector_length() {
        let a = [[0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8]];
        let b = [[0b00001111u8, 0b11110000u8, 0b00000000u8]];

        assert_eq!(a << 12, b);
    }

    #[test]
    fn shl_vector_length() {
        let a = [[0b00000000u8]];
        let b = [[]];

        assert_eq!(a << 8, b);
    }

    #[test]
    fn shl_more_than_vector_length() {
        let a = [[0b00000000u8]];
        let b = [[]];

        assert_eq!(a << 12, b);
    }

    #[test]
    fn shr_zero() {
        let a = [[0b00000000u8, 0b11111111u8]];
        let b = [[0b00000000u8, 0b11111111u8]];

        assert_eq!(a >> 0, b);
    }

    #[test]
    fn shr_less_than_vector_length() {
        let a = [[0b00000000u8, 0b11111111u8, 0b00000000u8, 0b00000000u8]];
        let b = [[0b00000000u8, 0b00001111u8, 0b11110000u8]];

        assert_eq!(a >> 12, b);
    }

    #[test]
    fn shr_vector_length() {
        let a = [[0b00000000u8]];
        let b = [[]];

        assert_eq!(a >> 8, b);
    }

    #[test]
    fn shr_more_than_vector_length() {
        let a = [[0b00000000u8]];
        let b = [[]];

        assert_eq!(a >> 12, b);
    }
}
