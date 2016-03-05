use std::iter;
use std::iter::{FromIterator};
use std::mem::size_of;
use std::fmt::{Debug, Formatter, Error};
use std::ops::{Index, IndexMut, BitAnd, BitOr, BitXor, Shl, Shr, Add, Range, RangeFrom, RangeTo, RangeFull};
use std::borrow::{Borrow, BorrowMut};
use num::traits::Zero;


pub trait TransposeBits: Sized {
    fn transpose_bits(self) -> Self;
}

macro_rules! vec_intrinsic_transpose_bits {
    ($t:ty) => {
        impl TransposeBits for Vec<$t> {
            fn transpose_bits(mut self) -> Vec<$t> {
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
                let mut j: $t = (8 * size_of::<$t>() as $t) / 2;
                let mut k: $t;

                let mut m: $t = (!0) >> j;
                let mut t: $t;

                while j != 0 {
                    k = 0;

                    while k < (8 * size_of::<$t>() as $t) {
                        t = (self[k as usize] ^ (self[(k|j) as usize] >> j)) & m;

                        self[k as usize] = self[k as usize] ^ t;
                        self[(k|j) as usize] = self[(k|j) as usize] ^ (t << j);

                        k = ((k | j) + 1) & (!j);
                    }

                    j = j >> 1;
                    m = m ^ (m << j);
                }

                self
            }
        }
    }
}
vec_intrinsic_transpose_bits!(u8);
vec_intrinsic_transpose_bits!(u16);
vec_intrinsic_transpose_bits!(u32);
vec_intrinsic_transpose_bits!(u64);


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
