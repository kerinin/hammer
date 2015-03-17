use bit_matrix::BitMatrix;

use std::collections::BitVec;

use db::value::{Value, Window, SubstitutionVariant, DeletionVariant, Hamming};

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
        let all = BitVec::from_elem(self.columns(), false);

        let shared_dimensions = self.data.iter()
            .zip(other.data.iter())
            .fold(all, |mut memo, (self_i, other_i)| {
                let xor_bytes_i = self_i.iter().zip(other_i.iter()).map(|(self_byte, other_byte)| {
                    self_byte ^ other_byte
                }).collect::<Vec<u8>>();

                memo.union(&BitVec::from_bytes(xor_bytes_i.as_slice()));
                memo
            });

        // Return the count of values which were not shared between all rows
        shared_dimensions.iter().filter(|x| *x).count()
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

        let mut different_dimensions = BitVec::from_elem(self_value.columns(), false);

        // Faster than building the bitmap and XOR-ing...
        if self_deleted_index != other_deleted_index {
            different_dimensions.set(self_value.columns() - self_deleted_index - 1, true);
            different_dimensions.set(other_value.columns() - other_deleted_index - 1, true);
        }

        for i in 0..self_value.rows() {
            let xor_bytes_i = self_value.data[i].iter().zip(other_value.data[i].iter()).map(|(self_byte, other_byte)| {
                self_byte ^ other_byte
            }).collect::<Vec<u8>>();

            different_dimensions.union(&BitVec::from_bytes(xor_bytes_i.as_slice()));
        }

        different_dimensions.iter().filter(|x| *x).count()
    }
    
    fn hamming_lte(&self, other: &(BitMatrix, usize), bound: usize) -> bool {
        let (self_value, self_deleted_index) = self.clone();
        let (other_value, other_deleted_index) = other.clone();

        let mut different_dimensions = BitVec::from_elem(self_value.columns(), false);

        // Faster than building the bitmap and XOR-ing...
        if self_deleted_index != other_deleted_index {
            different_dimensions.set(self_value.columns() - self_deleted_index - 1, true);
            different_dimensions.set(other_value.columns() - other_deleted_index - 1, true);
        }

        for i in 0..self_value.rows() {
            let xor_bytes_i = self_value.data[i].iter().zip(other_value.data[i].iter()).map(|(self_byte, other_byte)| {
                self_byte ^ other_byte
            }).collect::<Vec<u8>>();

            different_dimensions.union(&BitVec::from_bytes(xor_bytes_i.as_slice()));
            if different_dimensions.iter().filter(|x| *x).count() > bound {
                return true;
            }
        }

        different_dimensions.iter().filter(|x| *x).count() > bound
    }
}

#[cfg(test)]
mod test {
    extern crate quickcheck;
    use self::quickcheck::quickcheck;

    use bit_matrix::{BitMatrix, AsBitMatrix};

    use db::value::{Value, Window, SubstitutionVariant, DeletionVariant, Hamming};

    #[test]
    fn hamming_triangle_inequality() {
        fn prop(a: u64, b: u64, c:u64) -> quickcheck::TestResult {
            let a_bm = a.as_bit_matrix();
            let b_bm = b.as_bit_matrix();
            let c_bm = c.as_bit_matrix();
                
            quickcheck::TestResult::from_bool(
                a_bm.clone().hamming(&c_bm) <= (a_bm.clone().hamming(&b_bm) + b_bm.clone().hamming(&c_bm))
                )
        }
        quickcheck(prop as fn(u64, u64, u64) -> quickcheck::TestResult);
    }

    #[test]
    fn window_min_start_and_finish() {
        let a = BitMatrix::new(vec![vec![0b10000001u8]]);
        let b = BitMatrix::new(vec![vec![0b00000001u8]]);

        assert_eq!(a.window(0,1), b);
    }

    #[test]
    fn window_max_start() {
        let a = BitMatrix::new(vec![vec![0b10000001u8]]);
        let b = BitMatrix::new(vec![vec![0b00000001u8]]);

        assert_eq!(a.window(7,1), b);
    }

    #[test]
    fn window_min_start_and_max_finish() {
        let a = BitMatrix::new(vec![vec![0b10000001u8]]);
        let b = BitMatrix::new(vec![vec![0b10000001u8]]);

        assert_eq!(a.window(0,8), b);
    }

    #[test]
    fn window_n_start_and_max_finish() {
        let a = BitMatrix::new(vec![vec![0b11000011u8]]);
        let b = BitMatrix::new(vec![vec![0b01100001u8]]);

        assert_eq!(a.window(1,7), b);
    }

    #[test]
    fn window_min_start_and_n_finish() {
        let a = BitMatrix::new(vec![vec![0b11000011u8]]);
        let b = BitMatrix::new(vec![vec![0b01000011u8]]);

        assert_eq!(a.window(0,7), b);
    }

    #[test]
    fn window_n_start_and_n_finish() {
        let a = BitMatrix::new(vec![vec![0b11111000u8]]);
        let b = BitMatrix::new(vec![vec![0b00000011u8]]);

        assert_eq!(a.window(3,2), b);
    }

    #[test]
    fn permutation() {
        let a = BitMatrix::new(vec![vec![0b00000000u8]]);
        let expected = vec![
            BitMatrix::new(vec![vec![0b00000001u8]]),
            BitMatrix::new(vec![vec![0b00000010u8]]),
            BitMatrix::new(vec![vec![0b00000100u8]]),
            BitMatrix::new(vec![vec![0b00001000u8]]),
        ];

        assert_eq!(a.substitution_variants(4), expected);
    }

    #[test]
    fn hamming_zero() {
        let a = BitMatrix::new(vec![vec![0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![0b00000000u8]]);

        assert_eq!(a.hamming(&b), 0);
    }

    #[test]
    fn hamming_one() {
        let a = BitMatrix::new(vec![vec![0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![0b10000000u8]]);
        let c = BitMatrix::new(vec![vec![0b00000001u8]]);
        let d = BitMatrix::new(vec![vec![0b00010000u8]]);

        assert_eq!(a.hamming(&b), 1);
        assert_eq!(a.hamming(&c), 1);
        assert_eq!(a.hamming(&d), 1);
    }

    #[test]
    fn hamming_max() {
        let a = BitMatrix::new(vec![vec![0b00000000u8]]);
        let b = BitMatrix::new(vec![vec![0b11111111u8]]);

        assert_eq!(a.hamming(&b), 8);
    }


    #[test]
    fn substitution_variants() {
        let a = BitMatrix::new(vec![vec![0b00000000u8, 0b00000000u8]]);
        let expected = vec![
            BitMatrix::new(vec![vec![0b00000000u8, 0b00000001u8]]),
            BitMatrix::new(vec![vec![0b00000000u8, 0b00000010u8]]),
            BitMatrix::new(vec![vec![0b00000000u8, 0b00000100u8]]),
            BitMatrix::new(vec![vec![0b00000000u8, 0b00001000u8]]),
            BitMatrix::new(vec![vec![0b00000000u8, 0b00010000u8]]),
            BitMatrix::new(vec![vec![0b00000000u8, 0b00100000u8]]),
            BitMatrix::new(vec![vec![0b00000000u8, 0b01000000u8]]),
            BitMatrix::new(vec![vec![0b00000000u8, 0b10000000u8]]),
            BitMatrix::new(vec![vec![0b00000001u8, 0b00000000u8]]),
            BitMatrix::new(vec![vec![0b00000010u8, 0b00000000u8]]),
        ];

        assert_eq!(a.substitution_variants(10), expected);
    }

    #[test]
    fn deletion_variants() {
        let a = BitMatrix::new(vec![vec![0b00000000u8, 0b00000000u8]]);
        let expected = vec![
            (BitMatrix::new(vec![vec![0b00000000u8, 0b00000001u8]]), 0usize),
            (BitMatrix::new(vec![vec![0b00000000u8, 0b00000010u8]]), 1usize),
            (BitMatrix::new(vec![vec![0b00000000u8, 0b00000100u8]]), 2usize),
            (BitMatrix::new(vec![vec![0b00000000u8, 0b00001000u8]]), 3usize),
            (BitMatrix::new(vec![vec![0b00000000u8, 0b00010000u8]]), 4usize),
            (BitMatrix::new(vec![vec![0b00000000u8, 0b00100000u8]]), 5usize),
            (BitMatrix::new(vec![vec![0b00000000u8, 0b01000000u8]]), 6usize),
            (BitMatrix::new(vec![vec![0b00000000u8, 0b10000000u8]]), 7usize),
            (BitMatrix::new(vec![vec![0b00000001u8, 0b00000000u8]]), 8usize),
            (BitMatrix::new(vec![vec![0b00000010u8, 0b00000000u8]]), 9usize),
        ];

        assert_eq!(a.deletion_variants(10), expected);
    }

    #[test]
    fn deletion_hamming_equal_usize_u8() {
        let a = (BitMatrix::new(vec![vec![0b11111111u8]]), 0usize);
        let b = (BitMatrix::new(vec![vec![0b11111111u8]]), 0usize);

        let c = (BitMatrix::new(vec![vec![0b00000001u8]]), 0usize);
        let d = (BitMatrix::new(vec![vec![0b00000001u8]]), 0usize);

        assert_eq!(a.hamming(&b), 0);
        assert_eq!(c.hamming(&d), 0);
    }

    #[test]
    fn deletion_hamming_binary_unequal_usize_u8() {
        let a = (BitMatrix::new(vec![vec![0b11111111u8]]), 0usize);
        let b = (BitMatrix::new(vec![vec![0b01111111u8]]), 0usize);

        let c = (BitMatrix::new(vec![vec![0b10000001u8]]), 0usize);
        let d = (BitMatrix::new(vec![vec![0b00000001u8]]), 0usize);

        assert_eq!(a.hamming(&b), 1);
        assert_eq!(c.hamming(&d), 1);
    }

    #[test]
    fn deletion_hamming_deleted_unequal_usize_u8() {
        let a = (BitMatrix::new(vec![vec![0b11111111u8]]), 0usize);
        let b = (BitMatrix::new(vec![vec![0b11111111u8]]), 1usize);

        let c = (BitMatrix::new(vec![vec![0b00000001u8]]), 0usize);
        let d = (BitMatrix::new(vec![vec![0b00000010u8]]), 1usize);

        assert_eq!(a.hamming(&b), 2);
        assert_eq!(c.hamming(&d), 2);
    }

    #[test]
    fn deletion_hamming_binary_and_deleted_unequal_usize_u8() {
        let a = (BitMatrix::new(vec![vec![0b11111111u8]]), 0usize);
        let b = (BitMatrix::new(vec![vec![0b01111111u8]]), 1usize);

        let c = (BitMatrix::new(vec![vec![0b10000001u8]]), 0usize);
        let d = (BitMatrix::new(vec![vec![0b00000010u8]]), 1usize);

        assert_eq!(a.hamming(&b), 3);
        assert_eq!(c.hamming(&d), 3);
    }

}
