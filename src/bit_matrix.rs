use std::fmt::{Debug, Formatter, Error};
use std::ops::{Index, IndexMut, BitAnd, BitOr, BitXor, Shl, Shr};

/// A matrix of bits
///
/// Shift operations expect (row, column) values.  For column shifts "right" 
/// means "down" and "left" means "up".  In other words:
/// ```
/// let bit_matrix == [0, 1, 2, 3, 4, 5, 6, 7];
/// let n = 1;
/// let i = 1;
///
/// assert_eq!(bit_matrix[i] << n, (bit_matrix << (n, 0))[i]);
/// assert_eq!(bit_matrix[(i+n) % 8], (bit_matrix << (0, n))[i]);
/// ```
pub trait BitMatrix: 
Sized + 
BitAnd + 
BitOr + 
BitXor + 
Shl<(usize, usize)> + 
Shr<(usize, usize)> {
    fn count_ones(&self) -> Vec<u32>;

    fn transpose(self) -> Self;
}

macro_rules! intrinsic_matrix {
    ($t:ident, $u:ty, $n:expr) => {
        #[derive(Eq)]
        pub struct $t([$u; $n]);

        impl $t {
            pub fn from_elem(e: bool) -> $t {
                if e {
                    $t([!0; $n])
                } else {
                    $t([0; $n])
                }
            }

            pub fn new(v: [$u; $n]) -> $t {
                $t(v)
            }
        }

        impl Index<usize> for $t {
            type Output = $u;

            fn index(&self, index: usize) -> &$u {
                &self.0[index]
            }
        }

        impl IndexMut<usize> for $t {
            fn index_mut(&mut self, index: usize) -> &mut $u {
                &mut self.0[index]
            }
        }

        impl Debug for $t {
            fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                write!(f, "{:?}", self.0.iter().map(|n| format!("{:08b}", n)).collect::<Vec<String>>())
            }
        }

        impl Clone for $t {
            fn clone(&self) -> $t {
                let mut c = [0; $n];
                for i in 0..$n {
                    c[i] = self.0[i].clone();
                }
                $t(c)
            }
        }

        impl PartialEq for $t {
            fn eq(&self, other: &$t) -> bool {
                for i in 0..$n {
                    if self.0[i] != other.0[i] {
                        return false
                    }
                }
                return true
            }
        }

        impl BitMatrix for $t {
            fn count_ones(&self) -> Vec<u32> {
                let mut ones = Vec::with_capacity($n);
                for n in self.0.iter() {
                    ones.push(n.count_ones());
                }
                ones
            }

            fn transpose(mut self) -> $t {
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
                let mut j: $u = $n / 2;
                let mut k: $u;

                let mut m: $u = (!0) >> j;
                let mut t: $u;

                while j != 0 {
                    k = 0;

                    while k < $n {
                        t = (self.0[k as usize] ^ (self.0[(k|j) as usize] >> j)) & m;

                        self.0[k as usize] = self.0[k as usize] ^ t;
                        self.0[(k|j) as usize] = self.0[(k|j) as usize] ^ (t << j);

                        k = ((k | j) + 1) & (!j);
                    }

                    j = j >> 1;
                    m = m ^ (m << j);
                }

                self
            }
        }

        impl BitAnd for $t {
            type Output = $t;

            fn bitand(mut self, rhs: $t) -> $t {
                for i in 0..$n {
                    self.0[i] = self.0[i] & rhs.0[i]
                }
                self
            }
        }

        impl BitOr for $t {
            type Output = $t;

            fn bitor(mut self, rhs: $t) -> $t {
                for i in 0..$n {
                    self.0[i] = self.0[i] | rhs.0[i]
                }
                self
            }
        }

        impl BitXor for $t {
            type Output = $t;

            fn bitxor(mut self, rhs: $t) -> $t {
                for i in 0..$n {
                    self.0[i] = self.0[i] ^ rhs.0[i]
                }
                self
            }
        }

        impl Shl<(usize, usize)> for $t {
            type Output = $t;

            fn shl(mut self, (row_shift, column_shift): (usize, usize)) -> $t {
                match (row_shift, column_shift) {
                    (0, 0) => self,
                    (0, c) => {
                        for (to, from) in (c..$n).enumerate() {
                            self.0[to] = self.0[from];
                        }
                        for i in ($n-c)..$n {
                            self.0[i] = 0;
                        }
                        self
                    },
                    (r, 0) => {
                        for i in 0..$n {
                            self.0[i] = self.0[i] << r;
                        }
                        self
                    },
                    (r, c) => {
                        for (to, from) in (c..$n).enumerate() {
                            self.0[to] = self.0[from] << r;
                        }
                        for i in ($n-c)..$n {
                            self.0[i] = 0;
                        }
                        self
                    },
                }
            }
        }

        impl Shr<(usize, usize)> for $t {
            type Output = $t;

            fn shr(mut self, (row_shift, column_shift): (usize, usize)) -> $t {
                match (row_shift, column_shift) {
                    (0, 0) => self,
                    (0, c) => {
                        for (from, to) in (c..$n).enumerate().rev() {
                            self.0[to] = self.0[from];
                        }
                        for i in 0..c {
                            self.0[i] = 0;
                        }
                        self
                    },
                    (r, 0) => {
                        for i in 0..$n {
                            self.0[i] = self.0[i] >> r;
                        }
                        self
                    },
                    (r, c) => {
                        for (from, to) in (c..$n).enumerate().rev() {
                            self.0[to] = self.0[from] >> r;
                        }
                        for i in 0..c {
                            self.0[i] = 0;
                        }
                        self
                    },
                }
            }
        }
    }
}
intrinsic_matrix!(Matrix8, u8, 8);
intrinsic_matrix!(Matrix16, u16, 16);
intrinsic_matrix!(Matrix32, u32, 32);
intrinsic_matrix!(Matrix64, u64, 64);

#[cfg(test)] 
mod test {
    extern crate rand;
    extern crate quickcheck;

    use std;
    use self::quickcheck::quickcheck;

    use bit_matrix::{Matrix8, BitMatrix};

    #[test]
    fn transpose_sanity() {
        let v = Matrix8::new([0, 0, 0, 0, 0, 0, 1, std::u8::MAX]);
        let e = Matrix8::new([1, 1, 1, 1, 1, 1, 1, 3]);

        assert_eq!(v.transpose(), e);
    }

    #[test]
    fn transpose_identity() {
        fn prop(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8, h: u8) -> quickcheck::TestResult {
            let v = Matrix8::new([a, b, c, d, e, f, g, h]);

            quickcheck::TestResult::from_bool(v == v.clone().transpose().transpose())
        }
        quickcheck(prop as fn(u8, u8, u8, u8, u8, u8, u8, u8) -> quickcheck::TestResult);
    }

    #[test]
    fn clone_eq() {
        fn prop(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8, h: u8) -> quickcheck::TestResult {
            let v = Matrix8::new([a, b, c, d, e, f, g, h]);

            quickcheck::TestResult::from_bool(v == v.clone())
        }
        quickcheck(prop as fn(u8, u8, u8, u8, u8, u8, u8, u8) -> quickcheck::TestResult);
    }

    #[test]
    fn rotate_identity() {
        fn prop(mut x: usize, mut y: usize, a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8, h: u8) -> quickcheck::TestResult {
            x = 1 + (x % 7);
            y = 1 + (y % 7);

            let v = Matrix8::new([a, b, c, d, e, f, g, h]);

            let r1 = (v.clone() << (0, y)) | (v.clone() >> (0, 8 - y));
            let r2 = (r1.clone() >> (0, y)) | (r1.clone() << (0, 8 - y));
            let r3 = (r2.clone() << (x, 0)) | (r2.clone() >> (8 - x, 0));
            let r4 = (r3.clone() >> (x, 0)) | (r3.clone() << (8 - x, 0));

            quickcheck::TestResult::from_bool(v == r4)
        }
        quickcheck(prop as fn(usize, usize, u8, u8, u8, u8, u8, u8, u8, u8) -> quickcheck::TestResult);
    }
}
