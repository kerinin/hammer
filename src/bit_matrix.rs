use std;
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

pub struct Matrix32([u32; 32]);

impl Matrix32 {
    pub fn new(v: [u32; 32]) -> Matrix32 {
        Matrix32(v)
    }
}

impl BitMatrix for Matrix32 {
    fn count_ones(&self) -> Vec<u32> {
        let mut ones = Vec::with_capacity(32);
        for n in self.0.iter() {
            ones.push(n.count_ones());
        }
        ones
    }

    fn transpose(mut self) -> Matrix32{
        // Courtesy of http://www.hackersdelight.org/hdcodetxt/transpose32.c.txt
        let mut j = 16u32;
        let mut m = 0x0000ffffu32;
        let mut k;
        let mut t;
        while j != 0 {
            k = 0u32;
            while k < 32 {
                t = (self.0[k as usize] ^ (self.0[(k+j) as usize] >> j)) & m;

                self.0[k as usize] = self.0[k as usize] ^ t;
                self.0[(k+j) as usize] = self.0[(k+j) as usize] ^ (t << j);

                k = ((k | j) + 1) & (j ^ std::u32::MAX);
            }

            j = j >> 1;
            m = m ^ (m << j);
        }

        self
    }
}

impl BitAnd for Matrix32 {
    type Output = Matrix32;

    fn bitand(mut self, rhs: Matrix32) -> Matrix32 {
        for i in 0..32 {
            self.0[i] = self.0[i] & rhs.0[i]
        }
        self
    }
}

impl BitOr for Matrix32 {
    type Output = Matrix32;

    fn bitor(mut self, rhs: Matrix32) -> Matrix32 {
        for i in 0..32 {
            self.0[i] = self.0[i] | rhs.0[i]
        }
        self
    }
}

impl BitXor for Matrix32 {
    type Output = Matrix32;

    fn bitxor(mut self, rhs: Matrix32) -> Matrix32 {
        for i in 0..32 {
            self.0[i] = self.0[i] ^ rhs.0[i]
        }
        self
    }
}

impl Shl<(usize, usize)> for Matrix32 {
    type Output = Matrix32;

    fn shl(mut self, (row_shift, column_shift): (usize, usize)) -> Matrix32 {
        match (row_shift, column_shift) {
            (0, 0) => self,
            (0, c) => {
                for (from, to) in (0..c).enumerate() {
                    self.0[to] = self.0[from];
                }
                for i in c..32 {
                    self.0[i] = 0;
                }
                self
            },
            (r, 0) => {
                for i in 0..32 {
                    self.0[i] = self.0[i] << r;
                }
                self
            },
            (r, c) => {
                for (from, to) in (0..c).enumerate() {
                    self.0[to] = self.0[from] << r;
                }
                for i in c..32 {
                    self.0[i] = 0;
                }
                self
            },
        }
    }
}

impl Shr<(usize, usize)> for Matrix32 {
    type Output = Matrix32;

    fn shr(mut self, (row_shift, column_shift): (usize, usize)) -> Matrix32 {
        match (row_shift, column_shift) {
            (0, 0) => self,
            (0, c) => {
                for i in 0..c {
                    self.0[i] = 0;
                }
                for (from, to) in (c..32).enumerate() {
                    self.0[to] = self.0[from];
                }
                self
            },
            (r, 0) => {
                for i in 0..32 {
                    self.0[i] = self.0[i] >> r;
                }
                self
            },
            (r, c) => {
                for i in 0..c {
                    self.0[i] = 0;
                }
                for (from, to) in (c..32).enumerate() {
                    self.0[to] = self.0[from] >> r;
                }
                self
            },
        }
    }
}
