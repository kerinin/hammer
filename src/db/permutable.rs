use std::iter;
use std::vec::Vec;

/*
 * This is sort of annoying, but these traits aren't implemented for byte arrays,
 * and Rust doesn't allow implementations of non-owned traits for non-owned types.
 */
pub trait Permutable {
    fn p_bitxor(&self, &Self) -> Self;
    fn p_bitand(&self, &Self) -> Self;
    fn p_shl(&self, &usize) -> Self;
    fn p_shr(&self, &usize) -> Self;
}

impl Permutable for Vec<u8> {
    /*
     * Returns the result of bitxor-ing each byte of self and other.
     * If other is shorter than self, 0 will be used, if self is shorter than
     * other, the trailing bytes of other will be ignored
     */
    fn p_bitxor(&self, other: &Vec<u8>) -> Vec<u8> {
        let zero: &u8 = &0;
        let other_then_zero = other.iter().chain(iter::repeat(zero));

        return self.iter()
            .zip(other_then_zero)
            .map(|(self_byte, other_byte)| self_byte.clone() ^ *other_byte ) // bitxor
            .collect::<Vec<u8>>();
    }

    /*
     * Returns the result of bitand-ing each byte of self and other.
     * If other is shorter than self, self will be truncated to the same length.
     * If self is shorter than other, the trailing bytes of other will be ignored.
     */
    fn p_bitand(&self, other: &Vec<u8>) -> Vec<u8> {

        return self.iter()
            .zip(other.iter())
            .map(|(self_byte, other_byte)| self_byte.clone() & *other_byte ) // bitand
            .collect::<Vec<u8>>();
    }

    /*
     * Returns a new byte array with RHS bits removed from the left side, and
     * pads the left-most byte with zeros (if necessary)
     */
    fn p_shl(&self, rhs: &usize) -> Vec<u8> {
        if rhs == &0 { return self.clone(); }

        let to_drop = *rhs / 8;
        let to_shift = *rhs % 8;
        let to_unshift = 8 - to_shift;
        let mut out: Vec<u8> = vec![];

        // Drop elements containing only bits to be shifted off
        let mut iter = self.iter().skip(to_drop).peekable();

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

                            out.push(out_byte ^ bits_from_next_byte);
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

        return out;
    }

    /*
     * Returns a new byte array with RHS bits removed from the left side, and
     * pads the right-most byte with zeros (if necessary)
     */
    fn p_shr(&self, rhs: &usize) -> Vec<u8> {
        if rhs == &0 { return self.clone(); }

        let to_drop = *rhs / 8;
        let to_shift = *rhs % 8;
        let to_unshift = 8 - to_shift;
        let mut out: Vec<u8> = vec![];

        // Drop elements containing only bits to be shifted off
        let mut iter = self.iter().rev().skip(to_drop).peekable();

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

        return out;
    }
}

impl Permutable for usize {
    fn p_bitxor(&self, other: &usize) -> usize {
        self ^ other
    }

    fn p_bitand(&self, other: &usize) -> usize {
        self & other
    }

    fn p_shl(&self, rhs: &usize) -> usize {
        self << rhs
    }

    fn p_shr(&self, rhs: &usize) -> usize {
        self >> rhs
    }
}

#[cfg(test)]
mod test {
    use db::permutable::Permutable;

    #[test]
    fn bitxor_equally_sized_vectors() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let c = vec![0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8];

        assert_eq!(a.p_bitxor(&b), c);
    }

    #[test]
    fn bitxor_left_vector_longer() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let c = vec![0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];

        assert_eq!(a.p_bitxor(&b), c);
    }

    #[test]
    fn bitxor_right_vector_longer() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let c = vec![0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8];

        assert_eq!(a.p_bitxor(&b), c);
    }

    #[test]
    fn bitand_equally_sized_vectors() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let c = vec![0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8];

        assert_eq!(a.p_bitand(&b), c);
    }

    #[test]
    fn bitand_left_vector_longer() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let c = vec![0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8];

        assert_eq!(a.p_bitand(&b), c);
    }

    #[test]
    fn bitand_right_vector_longer() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let c = vec![0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8];

        assert_eq!(a.p_bitand(&b), c);
    }

    #[test]
    fn shl_zero() {
        let a = vec![0b00000000u8, 0b11111111u8];
        let b = vec![0b00000000u8, 0b11111111u8];

        assert_eq!(a.p_shl(&0), b);
    }

    #[test]
    fn shl_less_than_vector_length() {
        let a = vec![0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b00001111u8, 0b11110000u8, 0b00000000u8];

        assert_eq!(a.p_shl(&12), b);
    }

    #[test]
    fn shl_vector_length() {
        let a = vec![0b00000000u8];
        let b = vec![];

        assert_eq!(a.p_shl(&8), b);
    }

    #[test]
    fn shl_more_than_vector_length() {
        let a = vec![0b00000000u8];
        let b = vec![];

        assert_eq!(a.p_shl(&12), b);
    }

    #[test]
    fn shr_zero() {
        let a = vec![0b00000000u8, 0b11111111u8];
        let b = vec![0b00000000u8, 0b11111111u8];

        assert_eq!(a.p_shr(&0), b);
    }

    #[test]
    fn shr_less_than_vector_length() {
        let a = vec![0b00000000u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let b = vec![0b00000000u8, 0b00001111u8, 0b11110000u8];

        assert_eq!(a.p_shr(&12), b);
    }

    #[test]
    fn shr_vector_length() {
        let a = vec![0b00000000u8];
        let b = vec![];

        assert_eq!(a.p_shr(&8), b);
    }

    #[test]
    fn shr_more_than_vector_length() {
        let a = vec![0b00000000u8];
        let b = vec![];

        assert_eq!(a.p_shr(&12), b);
    }
}
