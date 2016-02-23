use std::clone::*;
use std::iter::*;

pub struct DeletionIter<T> {
    // The original value, which shouldn't be modified
    source: T,
    // Iteration cursor
    index: usize,
    // The number of dimensions to iterate over
    dimensions: usize,
}

impl<T> DeletionIter<T> {
    pub fn new(v: T, dimensions: usize) -> Self {
        DeletionIter {
            source: v,
            index: 0,
            dimensions: dimensions,
        }
    }
}

impl Iterator for DeletionIter<u8> {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<(u8, u8)> {
        if self.index >= self.dimensions {
            None
        } else {
            let next_value = self.source.clone() | (1u8 << self.index);

            self.index += 1;
            Some((next_value, self.index as u8))
        }
    }
}

impl Iterator for DeletionIter<usize> {
    type Item = (usize, u8);

    fn next(&mut self) -> Option<(usize, u8)> {
        if self.index >= self.dimensions {
            None
        } else {
            let next_value = self.source.clone() | (1usize << self.index);

            self.index += 1;
            Some((next_value, self.index as u8))
        }
    }
}

impl Iterator for DeletionIter<u64> {
    type Item = (u64, u8);

    fn next(&mut self) -> Option<(u64, u8)> {
        if self.index >= self.dimensions {
            None
        } else {
            let next_value = self.source.clone() | (1u64 << self.index);

            self.index += 1;
            Some((next_value, self.index as u8))
        }
    }
}

#[cfg(test)] 
mod test {
    extern crate quickcheck;

    use std::collections::*;

    use self::quickcheck::quickcheck;

    use db::deletion::{DeletionVariant};

    #[test]
    fn variants_compact_u64() {
        // Test that the variants don't contain duplicates. (This could happen
        // from hash collisions, but that should be exceedingly rare)
        fn prop(a: u64) -> quickcheck::TestResult {
            let variants = a.deletion_variants(64);

            for (i, variant) in variants.enumerate() {
                if a.deletion_variants(64).take(i).any(|v| v == variant) {
                    return quickcheck::TestResult::failed();
                }
            }
            return quickcheck::TestResult::passed();
        }
        quickcheck(prop as fn(u64) -> quickcheck::TestResult);
    }

    #[test]
    fn hamming_1_variants_u64() {
        // Test that the intersection of the deletion variants for two values 
        // with hamming distance 1 is not empty.
        fn prop(a: u64, index: usize) -> quickcheck::TestResult {
            let flip = 1u64 << index % 64;
            let b = a ^ flip;

            let a_variants = a.deletion_variants(64).fold(HashSet::new(), |mut s, v| { s.insert(v); s });
            let b_variants = b.deletion_variants(64).fold(HashSet::new(), |mut s, v| { s.insert(v); s });

            if a_variants.intersection(&b_variants).count() == 0 {
                return quickcheck::TestResult::failed();
            }

            return quickcheck::TestResult::passed();
        }
        quickcheck(prop as fn(u64, usize) -> quickcheck::TestResult);
    }

    #[test]
    fn variants_compact_vec_u8() {
        // Test that the variants don't contain duplicates. (This could happen
        // from hash collisions, but that should be exceedingly rare)
        fn prop(a: Vec<u8>) -> quickcheck::TestResult {
            if a.len() == 0 {
                return quickcheck::TestResult::discard()
            }

            let variants = a.deletion_variants(a.len());

            for (i, variant) in variants.enumerate() {
                if a.deletion_variants(a.len()).take(i).any(|v| v == variant) {
                    return quickcheck::TestResult::failed();
                }
            }
            return quickcheck::TestResult::passed();
        }
        quickcheck(prop as fn(Vec<u8>) -> quickcheck::TestResult);
    }

    #[test]
    fn hamming_1_variants_vec_u8() {
        // Test that the intersection of the deletion variants for two values 
        // with hamming distance 1 is not empty.
        fn prop(a: Vec<u8>, index: usize) -> quickcheck::TestResult {
            if a.len() == 0 {
                return quickcheck::TestResult::discard()
            }

            let mut b = a.clone();
            let offset = index % b.len();
            b[offset] = b[offset] ^ 1u8;

            let a_variants = a.deletion_variants(a.len()).fold(HashSet::new(), |mut s, v| { s.insert(v); s });
            let b_variants = b.deletion_variants(b.len()).fold(HashSet::new(), |mut s, v| { s.insert(v); s });

            if a_variants.intersection(&b_variants).count() == 0 {
                return quickcheck::TestResult::failed();
            }

            return quickcheck::TestResult::passed();
        }
        quickcheck(prop as fn(Vec<u8>, usize) -> quickcheck::TestResult);
    }

    #[test]
    fn test_deletion_variants_u8() {
        let a = 0b00000000u8;
        let expected = vec![
            (0b00000001u8, 1u8),
            (0b00000010u8, 2u8),
            (0b00000100u8, 3u8),
            (0b00001000u8, 4u8),
        ];

        assert_eq!(a.deletion_variants(4).collect::<Vec<(u8, u8)>>(), expected);
    }

    #[test]
    fn test_deletion_variants_usize() {
        let a = 0b00000000usize;
        let expected = vec![
            (0b00000001usize, 1u8),
            (0b00000010usize, 2u8),
            (0b00000100usize, 3u8),
            (0b00001000usize, 4u8),
        ];

        assert_eq!(a.deletion_variants(4).collect::<Vec<(usize, u8)>>(), expected);
    }
}
