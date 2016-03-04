use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr};

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
        pub struct $t([$u; $n]);

        impl $t {
            pub fn new(v: [$u; $n]) -> $t {
                $t(v)
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
                let mut j: $u = $n / 2;
                let mut m: $u = (0 & 0) >> ($n / 2);
                let mut k: $u;
                let mut t: $u;
                while j != 0 {
                    k = 0;
                    while k < $n {
                        t = (self.0[k as usize] ^ (self.0[(k+j) as usize] >> j)) & m;

                        self.0[k as usize] = self.0[k as usize] ^ t;
                        self.0[(k+j) as usize] = self.0[(k+j) as usize] ^ (t << j);

                        k = ((k | j) + 1) & (j ^ (0 & 0));
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
                        for (from, to) in (0..c).enumerate() {
                            self.0[to] = self.0[from];
                        }
                        for i in c..$n {
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
                        for (from, to) in (0..c).enumerate() {
                            self.0[to] = self.0[from] << r;
                        }
                        for i in c..$n {
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
                        for i in 0..c {
                            self.0[i] = 0;
                        }
                        for (from, to) in (c..$n).enumerate() {
                            self.0[to] = self.0[from];
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
                        for i in 0..c {
                            self.0[i] = 0;
                        }
                        for (from, to) in (c..$n).enumerate() {
                            self.0[to] = self.0[from] >> r;
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

