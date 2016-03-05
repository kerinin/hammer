use std::iter;
use std::iter::{FromIterator};
use std::mem::size_of;
use std::fmt::{Debug, Formatter, Error};
use std::ops::{Index, IndexMut, BitAnd, BitOr, BitXor, Shl, Shr, Mul, Add, Range, RangeFrom, RangeTo, RangeFull, Not};
use std::borrow::{Borrow, BorrowMut};
use num::traits::Zero;


pub trait TransposeBits: Sized {
    fn transpose_bits(self) -> Self;
}

impl<T> TransposeBits for Vec<T> where
T: BitXor<Output=T>,
T: BitAnd<Output=T>,
T: Not<Output=T>,
T: Shl<usize, Output=T>,
T: Shr<usize, Output=T>,
T: Mul<T>,
T: Zero,
T: Clone,
{
    fn transpose_bits(mut self) -> Vec<T> {
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

        self
    }
}


#[cfg(test)] 
mod test {
    extern crate rand;
    extern crate quickcheck;

    use std;
    use self::quickcheck::quickcheck;

    use bit_matrix::{TransposeBits};

    #[test]
    fn transpose_sanity() {
        let v = vec![0, 0, 0, 0, 0, 0, 1, std::u8::MAX];
        let e = vec![1, 1, 1, 1, 1, 1, 1, 3];

        assert_eq!(v.transpose_bits(), e);
    }

    #[test]
    fn transpose_identity() {
        fn prop(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8, h: u8) -> quickcheck::TestResult {
            let v = vec![a, b, c, d, e, f, g, h];

            quickcheck::TestResult::from_bool(v == v.clone().transpose_bits().transpose_bits())
        }
        quickcheck(prop as fn(u8, u8, u8, u8, u8, u8, u8, u8) -> quickcheck::TestResult);
    }
}
