use std::clone::*;
use std::iter::*;
use std::hash::*;
use std::default::*;

pub trait DeletionVariant: Sized {
    type Iter: Iterator<Item = Self>;

    /// Returns an array of all possible deletion variants of `self`
    ///
    /// A "deletion variant" as defined in
    /// [Zhang](http://www.cse.unsw.edu.au/~weiw/files/SSDBM13-HmSearch-Final.pdf)
    /// is a value obtained by substituting a "deletion marker" for a single 
    /// dimension of a value.
    ///
    fn deletion_variants(&self, dimensions: usize) -> <Self as DeletionVariant>::Iter;
}

pub struct DeletionVariantIter<T> {
    // The original value, which shouldn't be modified
    source: T,
    // Iteration cursor
    index: usize,
    // The number of dimensions to iterate over
    dimensions: usize,
}

impl<T> DeletionVariantIter<T> {
    pub fn new(v: T, dimensions: usize) -> Self {
        DeletionVariantIter {
            source: v,
            index: 0,
            dimensions: dimensions,
        }
    }
}

impl<T> DeletionVariant for T where 
T: Clone,
DeletionVariantIter<T>: Iterator<Item = T>
{
    type Iter = DeletionVariantIter<T>;

    fn deletion_variants(&self, dimensions: usize) -> DeletionVariantIter<T> {
        DeletionVariantIter::new(self.clone(), dimensions)
    }
}

impl Iterator for DeletionVariantIter<(u8, u8)> {
    type Item = (u8, u8);

    fn next(&mut self) -> Option<(u8, u8)> {
        if self.index >= self.dimensions {
            None
        } else {
            let (mut next_value, _) = self.source.clone();
            next_value = next_value | (1u8 << self.index);
            
            self.index += 1;
            Some((next_value, self.index as u8))
        }
    }
}

impl Iterator for DeletionVariantIter<(usize, u8)> {
    type Item = (usize, u8);

    fn next(&mut self) -> Option<(usize, u8)> {
        if self.index >= self.dimensions {
            None
        } else {
            let (mut next_value, _) = self.source.clone();
            next_value = next_value | (1usize << self.index);
            
            self.index += 1;
            Some((next_value, self.index as u8))
        }
    }
}

impl Iterator for DeletionVariantIter<(u64, u8)> {
    type Item = (u64, u8);

    fn next(&mut self) -> Option<(u64, u8)> {
        if self.index >= self.dimensions {
            None
        } else {
            let (mut next_value, _) = self.source.clone();
            next_value = next_value | (1u64 << self.index);
            
            self.index += 1;
            Some((next_value, self.index as u8))
        }
    }
}

/*
impl<T> DeletionVariant for Vec<T> where
T: Hash,
Vec<T>: Clone,
XORDeletionVariantIter<Vec<T>>: Iterator,
{
    type Iter = XORDeletionVariantIter<Vec<T>>;

    fn deletion_variants(&self, dimensions: usize) -> XORDeletionVariantIter<Vec<T>> {
        XORDeletionVariantIter::new(self.clone(), dimensions)
    }
}
*/

pub struct XORDeletionVariantIter<T> {
    // XOR-ed hash of each dimension index & value in `source`
    source_hash: u64,
    // The original value, which shouldn't be modified
    source: T,
    // Iteration cursor
    index: usize,
    // The number of dimensions to iterate over
    dimensions: usize,
}

// NOTE: Consider parameterizing on the hasher state so we ensure the dimension
// hashes are always consistent
impl<T> XORDeletionVariantIter<Vec<T>>
where T: Hash,
    Vec<T>: Clone,
{
    pub fn new(v: Vec<T>, dimensions: usize) -> Self {
        let mut dv = XORDeletionVariantIter {
            source_hash: 0,
            source: v.clone(),
            index: 0,
            dimensions: dimensions,
        };
        for (i, v_i) in v.iter().enumerate() {
            let mut hasher: SipHasher = Default::default();
            v_i.hash(&mut hasher);
            i.hash(&mut hasher);
            dv.source_hash = dv.source_hash ^ hasher.finish();
        }
        dv
    }
}

impl Iterator for XORDeletionVariantIter<Vec<u8>> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.index >= self.dimensions {
            None
        } else {
            // NOTE: We're using the initial `source_hash` value as the deltion
            // marker becuase it means we don't have to XOR in the deltion marker
            // value.  We _may_ need to XOR in the deltion marker's index, IDK
            //
            if self.index > 0 {
                // Add the last index's hash back in
                let mut hasher: SipHasher = Default::default();
                self.source[self.index - 1].hash(&mut hasher);
                (self.index - 1).hash(&mut hasher);

                self.source_hash = self.source_hash ^ hasher.finish();
                
                // Remove the last index
                let mut hasher: SipHasher = Default::default();
                (self.index - 1).hash(&mut hasher);

                self.source_hash = self.source_hash ^ hasher.finish();
            }
            // Remove the current index's hash
            let mut hasher: SipHasher = Default::default();
            self.source[self.index].hash(&mut hasher);
            self.index.hash(&mut hasher);
            self.source_hash = self.source_hash ^ hasher.finish();
            self.index += 1;

            // Add the current index
            let mut hasher: SipHasher = Default::default();
            (self.index).hash(&mut hasher);

            self.source_hash = self.source_hash ^ hasher.finish();

            Some(self.source_hash.clone())
        }
    }
}

#[cfg(test)] 
mod test {
    extern crate quickcheck;

    use std::collections::*;

    use self::quickcheck::quickcheck;

    use db::deletion_variant::*;

    #[test]
    fn variants_compact_u64() {
        // Test that the variants don't contain duplicates. (This could happen
        // from hash collisions, but that should be exceedingly rare)
        fn prop(a: u64) -> quickcheck::TestResult {
            let variants = (a, 0u8).deletion_variants(64);

            for (i, variant) in variants.enumerate() {
                if (a, 0u8).deletion_variants(64).take(i).any(|v| v == variant) {
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

            let a_variants = (a, 0u8).deletion_variants(64).fold(HashSet::new(), |mut s, v| { s.insert(v); s });
            let b_variants = (b, 0u8).deletion_variants(64).fold(HashSet::new(), |mut s, v| { s.insert(v); s });

            if a_variants.intersection(&b_variants).count() == 0 {
                return quickcheck::TestResult::failed();
            }

            return quickcheck::TestResult::passed();
        }
        quickcheck(prop as fn(u64, usize) -> quickcheck::TestResult);
    }

    /*
    #[test]
    fn variants_compact_vec_u8() {
        // Test that the variants don't contain duplicates. (This could happen
        // from hash collisions, but that should be exceedingly rare)
        fn prop(a: Vec<u8>) -> quickcheck::TestResult {
            if a.len() == 0 {
                return quickcheck::TestResult::discard()
            }

            let variants = (a, 0u8).deletion_variants(a.len());

            for (i, variant) in variants.enumerate() {
                if (a, 0u8).deletion_variants(a.len()).take(i).any(|v| v == variant) {
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

            let a_variants = (a, 0u8).deletion_variants(a.len()).fold(HashSet::new(), |mut s, v| { s.insert(v); s });
            let b_variants = (b, 0u8).deletion_variants(b.len()).fold(HashSet::new(), |mut s, v| { s.insert(v); s });

            if a_variants.intersection(&b_variants).count() == 0 {
                return quickcheck::TestResult::failed();
            }

            return quickcheck::TestResult::passed();
        }
        quickcheck(prop as fn(Vec<u8>, usize) -> quickcheck::TestResult);
    }
    */

    #[test]
    fn test_deletion_variants_u8() {
        let a = 0b00000000u8;
        let expected = vec![
            (0b00000001u8, 1u8),
            (0b00000010u8, 2u8),
            (0b00000100u8, 3u8),
            (0b00001000u8, 4u8),
        ];

        assert_eq!((a, 0u8).deletion_variants(4).collect::<Vec<(u8, u8)>>(), expected);
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

        assert_eq!((a, 0u8).deletion_variants(4).collect::<Vec<(usize, u8)>>(), expected);
    }
}
