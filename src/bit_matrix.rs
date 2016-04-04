use std::mem::size_of;
use std::ops::{BitAnd, BitXor, Shl, Shr, Not};
use num::traits::Zero;


/// Trait of objects whose bits can be transposed
///
/// `BitTranspose` does a square-matrix transposition of bits, so for instance
///
/// ```ignore
/// # mod bit_matrix;
/// use bit_matrix::BitTranspose;
///
/// let a = vec![
///     0b11111111u8,
///     0b00000000u8,
///     0b00000000u8,
///     0b00000000u8,
///     0b00000000u8,
///     0b11000000u8,
///     0b11000000u8,
///     0b11000000u8,
///     ];
///
/// let b = vec![
///     0b10000111u8,
///     0b10000111u8,
///     0b10000000u8,
///     0b10000000u8,
///     0b10000000u8,
///     0b10000000u8,
///     0b10000000u8,
///     0b10000000u8,
///     ];
///
/// assert_eq!(a.bit_transpose(), b);
/// ```
pub trait BitTranspose: Sized {
    fn bit_transpose(mut self) -> Self {
        self.bit_transpose_assign();
        self
    }

    fn bit_transpose_assign(&mut self);
}

impl<T> BitTranspose for Vec<T> where
T: BitXor<Output=T>,
T: BitAnd<Output=T>,
T: Not<Output=T>,
T: Shl<usize, Output=T>,
T: Shr<usize, Output=T>,
T: Zero,
T: Clone,
{
    fn bit_transpose_assign(&mut self) {
        assert_eq!(self.len(), 8 * size_of::<T>());

        // Courtesy of http://www.hackersdelight.org/hdcodetxt/transpose32.c.txt
        //
        // void transpose32b(unsigned A[32]) {
        //    int j, k;
        //    unsigned m, t;
        //
        //    m = 0x0000FFFF;
        //    for (j = 16; j != 0; j = j >> 1, m = m ^ (m << j)) {
        //       for (k = 0; k < 32; k = (k + j + 1) & ~j) {
        //          t = (A[k] ^ (A[k+j] >> j)) & m;
        //          A[k] = A[k] ^ t;
        //          A[k+j] = A[k+j] ^ (t << j);
        //       }
        //    }
        // }
        let mut j: usize = (8 * size_of::<T>()) / 2;
        let mut k: usize;

        let mut m: T = (!<T as Zero>::zero()) >> j;
        let mut t: T;

        while j != 0 {
            k = 0;

            while k < (8 * size_of::<T>()) {
                t = (self[k].clone() ^ (self[k|j].clone() >> j)) & m.clone();

                self[k] = self[k].clone() ^ t.clone();
                self[k|j] = self[k|j].clone() ^ t << j;

                k = ((k | j) + 1) & (!j);
            }

            j = j >> 1;
            m = m.clone() ^ (m << j);
        }
    }
}


#[cfg(test)] 
mod test {
    extern crate rand;
    extern crate quickcheck;

    use self::quickcheck::quickcheck;

    use bit_matrix::{BitTranspose};

    #[test]
    fn transpose_identity() {
        fn prop(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8, h: u8) -> quickcheck::TestResult {
            let v = vec![a, b, c, d, e, f, g, h];

            quickcheck::TestResult::from_bool(v == v.clone().bit_transpose().bit_transpose())
        }
        quickcheck(prop as fn(u8, u8, u8, u8, u8, u8, u8, u8) -> quickcheck::TestResult);
    }
}
