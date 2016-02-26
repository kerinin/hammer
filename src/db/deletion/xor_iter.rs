use std::clone::*;
use std::iter::*;
use std::hash::*;
use std::default::*;

use db::deletion::{Dvec};

/// XORIter is an optimization for vectors
///
/// Typical deletion variants are created by cloning the base value, setting
/// the dimension being deleted to a 0-value and then returning a tuple of the
/// cloned value and the index of the deletion.  This requires allocating the 
/// full size of the source value for each variant.  For smaller values (ie u64)
/// this isn't a problem, but when working with large vectors it can quickly
/// become a bottleneck
///
/// Deletion variants are used as hash keys, so their literal value is irrelevant 
/// so long as distinct source values generate distinct deletion variant. This
/// provides the first optimization; the iterator starts by creating a hash of 
/// each vector element and its offset.  These hashes are XOR-ed together to 
/// create the `source_hash` which will be returned as the deletion variant. 
/// A "deleted" element's hash and index are not XOR-ed into the output, creating
/// a unique set of deletion variants for each source value.
///
/// The XOR operation is transitive and follows the following rule
///     A XOR (A XOR B) = B
/// This allows us to XOR together an arbitrary number of values, and "back out"
/// any single value from the result simply by doing XOR-ing it with the result.
/// This provides the second optimization: rather than keeping the hash values 
/// of each vector element in memory and recomputing the XOR result for each
/// variant, we simply compute the first deletion variant.  Subsequent variants
/// can be computed by "adding in" the last vector element and then "backing out"
/// the next vector element
///
/// XORIter is susceptable to hash collisions, but collisions
/// in this case don't affect the query's correctness and should have a 
/// trivial impact on performance
///
pub struct XORIter<T> {
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
impl<T> XORIter<Vec<T>> where 
T: Hash,
Vec<T>: Clone,
{
    pub fn new(v: Vec<T>, dimensions: usize) -> Self {
        let mut dv = XORIter {
            source_hash: 0,
            source: v.clone(),
            index: 1,
            dimensions: dimensions,
        };
        for (i, v_i) in v.iter().enumerate() {
            let mut hasher: SipHasher = Default::default();
            v_i.hash(&mut hasher);
            // start at index 1 to ensure that each element mutates the hash
            (i+1).hash(&mut hasher);
            dv.source_hash = dv.source_hash ^ hasher.finish();
        }
        dv
    }
}

impl Iterator for XORIter<Vec<u8>> {
    type Item = Dvec;

    fn next(&mut self) -> Option<Dvec> {
        if self.index > self.dimensions {
            None
        } else {
            // NOTE: Pretty sure this logic is fubar
            // NOTE: We're using the initial `source_hash` value as the deltion
            // marker becuase it means we don't have to XOR in the deltion marker
            // value.  We _may_ need to XOR in the deltion marker's index, IDK
            //
            let mut hasher: SipHasher = Default::default();

            if self.index > 1 {
                // Add the last index's hash back in
                self.source[self.index - 2].hash(&mut hasher);
                (self.index - 1).hash(&mut hasher);

                self.source_hash = self.source_hash ^ hasher.finish();
            }

            // Remove the current index's hash
            hasher = Default::default();
            self.source[self.index - 1].hash(&mut hasher);
            self.index.hash(&mut hasher);

            self.source_hash = self.source_hash ^ hasher.finish();
            self.index += 1;

            Some(self.source_hash.clone())
        }
    }
}
