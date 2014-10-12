use std::iter::Repeat;
use std::vec::Vec;

pub trait Permutable<V, I, Result> {
    fn bitxor(&self, rhs: &V) -> Result;
    fn bitand(&self, rhs: &V) -> Result;
    fn shl(&self, rhs: &I) -> Result;
    fn shr(&self, rhs: &I) -> Result;
}

impl Permutable<Vec<u8>, uint, Vec<u8>> for Vec<u8> {
    /*
     * Returns the result of bitxor-ing each byte of self and other.
     * If other is shorter than self, 0 will be used, if self is shorter than
     * other, the trailing bytes of other will be ignored
     */
    fn bitxor(&self, other: &Vec<u8>) -> Vec<u8> {
        let zero: &u8 = &0;
        let other_then_zero = other.iter().chain(Repeat::new(zero));

        return self.iter()
            .zip(other_then_zero)
            .map(|(self_byte, other_byte)| self_byte.clone().bitxor(other_byte) )
            .collect::<Vec<u8>>();
    }

    /*
     * Returns the result of bitand-ing each byte of self and other.
     * If other is shorter than self, self will be truncated to the same length.
     * If self is shorter than other, the trailing bytes of other will be ignored.
     */
    fn bitand(&self, other: &Vec<u8>) -> Vec<u8> {

        return self.iter()
            .zip(other.iter())
            .map(|(self_byte, other_byte)| self_byte.clone().bitand(other_byte) )
            .collect::<Vec<u8>>();

        //let other_then_zero = other.iter().chain(Repeat::new(zero));

        //return self.iter()
        //    .zip(other_then_zero)
        //    .map(|(self_byte, other_byte)| self_byte.clone().bitand(other_byte) )
        //    .collect::<Vec<u8>>();
    }

    /*
     * Returns a new byte array with RHS bits removed from the left side, and
     * pads the left-most byte with zeros (if necessary)
     */
    fn shl(&self, rhs: &uint) -> Vec<u8> {
        let to_drop = rhs / 8;
        let to_shift = rhs % 8;
        let to_unshift = 8 - to_shift;
        let mut out: Vec<u8> = vec![];

        // Drop elements containing only bits to be shifted off
        let mut iter = self.iter().skip(to_drop).peekable();

        // If we don't need to shift any bits, we're done
        while to_shift > 0 {
            match iter.next() {

                Some(&this_byte) => {
                    // Shift some bits 
                    let out_byte = this_byte.shl(&to_shift);

                    match iter.peek() {

                        // There's another byte - shift some of its bits 
                        // into this byte
                        Some(&next_byte) => {
                            let bits_from_next_byte = &next_byte.shr(&to_unshift);

                            out.push(out_byte.bitxor(bits_from_next_byte));
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
    fn shr(&self, rhs: &uint) -> Vec<u8> {
        let to_drop = rhs / 8;
        let to_shift = rhs % 8;
        let to_unshift = 8 - to_shift;
        let mut out: Vec<u8> = vec![];

        // Drop elements containing only bits to be shifted off
        let mut iter = self.iter().rev().skip(to_drop).peekable();

        // If we don't need to shift any bits, we're done
        while to_shift > 0 {
            match iter.next() {

                Some(&this_byte) => {
                    // Shift some bits 
                    let out_byte = this_byte.shr(&to_shift);

                    match iter.peek() {

                        // There's another byte - shift some of its bits 
                        // into this byte
                        Some(&next_byte) => {
                            let bits_from_next_byte = &next_byte.shl(&to_unshift);

                            out.insert(0, out_byte.bitxor(bits_from_next_byte));
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

#[cfg(test)]
mod test {
    use super::{Permutable};

    #[test]
    fn bitxor_equally_sized_vectors() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let c = vec![0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8];

        assert_eq!(a.bitxor(&b), c);
    }

    #[test]
    fn bitxor_left_vector_longer() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let c = vec![0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];

        assert_eq!(a.bitxor(&b), c);
    }

    #[test]
    fn bitxor_right_vector_longer() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let c = vec![0b00000000u8, 0b11111111u8, 0b11111111u8, 0b00000000u8];

        assert_eq!(a.bitxor(&b), c);
    }

    #[test]
    fn bitand_equally_sized_vectors() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let c = vec![0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8];

        assert_eq!(a.bitand(&b), c);
    }

    #[test]
    fn bitand_left_vector_longer() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let c = vec![0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8];

        assert_eq!(a.bitand(&b), c);
    }

    #[test]
    fn bitand_right_vector_longer() {
        let a = vec![0b11111111u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b11111111u8, 0b11111111u8, 0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let c = vec![0b11111111u8, 0b00000000u8, 0b00000000u8, 0b00000000u8];

        assert_eq!(a.bitand(&b), c);
    }

    #[test]
    fn shl_less_than_vector_length() {
        let a = vec![0b00000000u8, 0b00000000u8, 0b11111111u8, 0b00000000u8];
        let b = vec![0b00001111u8, 0b11110000u8, 0b00000000u8];

        assert_eq!(a.shl(&12), b);
    }

    #[test]
    fn shl_vector_length() {
        let a = vec![0b00000000u8];
        let b = vec![];

        assert_eq!(a.shl(&8), b);
    }

    #[test]
    fn shl_more_than_vector_length() {
        let a = vec![0b00000000u8];
        let b = vec![];

        assert_eq!(a.shl(&12), b);
    }

    #[test]
    fn shr_less_than_vector_length() {
        let a = vec![0b00000000u8, 0b11111111u8, 0b00000000u8, 0b00000000u8];
        let b = vec![0b00000000u8, 0b00001111u8, 0b11110000u8];

        assert_eq!(a.shr(&12), b);
    }

    #[test]
    fn shr_vector_length() {
        let a = vec![0b00000000u8];
        let b = vec![];

        assert_eq!(a.shr(&8), b);
    }

    #[test]
    fn shr_more_than_vector_length() {
        let a = vec![0b00000000u8];
        let b = vec![];

        assert_eq!(a.shr(&12), b);
    }
}
