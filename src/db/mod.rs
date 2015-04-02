//! Hamming distance query database
//!
//! Given a query `Q`, returns the set of all values `[V]` within a given hamming 
//! distance from `Q`.
//!
//! Implements the [HmSearch](http://www.cse.unsw.edu.au/~weiw/files/SSDBM13-HmSearch-Final.pdf)
//! algorithm of Zhang et. al. with some modifications.
//!
//! Provides two variants, `SubstitutionDB` and `DeletionDB`.  `SubstitutionDB` 
//! operates by storing all permutations of indexed values, while `DeletionDB`
//! operates by storing all "deletion variants" of indexed values.  Given a set
//! of `D` dimensions taking one of `V` values, the former has query time 
//! complexity of `O(1)` and storage complexity `O(D*V)`, while the latter has
//! query time complexity `O(D)` and storage complexity `O(D)`.  In other words,
//! use `SubstitutionDB` to store binary vectors, and `DeletionDB` to store 
//! vectors of anything more complex.
//!
//! # Examples
//!
//! ```ignore
//! let dimensions = 64;
//! let tolerance = 1;
//! let mut db: SubstitutionDB<usize> = SubstitutionDB::new(dimensions, tolerance)
//!
//! db.insert(0);
//! db.insert(1);
//! db.insert(3);
//! db.insert(7);
//! db.insert(1000);
//!
//! let results = db.get(&0).iter().collect();
//! assert_eq!(results, vec![0,1,3,7]);
//! ```

mod substitution_db;
mod deletion_db;
mod value;
mod hash_map_set;
mod result_accumulator;
mod deletion_variant;
mod substitution_variant;
mod hashing;

mod bench; // Uncomment to get benchmarks to run

use std::hash;
use std::cmp;
use std::clone;
use std::rc::*;
use std::collections::HashSet;

use db::substitution_db::SubstitutionPartition;
use db::hash_map_set::HashMapSet;

/// Abstract interface for Hamming distance databases
///
pub trait Database<V: Value> {
    fn new(dimensions: usize, tolerance: usize) -> Self;
    fn get(&self, key: &V) -> Option<HashSet<V>>;
    fn insert(&mut self, key: V) -> bool;
    fn remove(&mut self, key: &V) -> bool;
}

/// HmSearch Database using substitution variants
///
pub struct SubstitutionDB<V> where
    V: Value + Window + SubstitutionVariant,
    <<V as SubstitutionVariant>::Iter as Iterator>::Item: cmp::Eq + hash::Hash + clone::Clone,
{
    dimensions: usize,
    tolerance: usize,
    partition_count: usize,
    partitions: Vec<SubstitutionPartition<V>>,
}

pub struct DeletionPartition<V> where
    V: Value + DeletionVariant,
    <<V as DeletionVariant>::Iter as Iterator>::Item: cmp::Eq + hash::Hash + clone::Clone,
{
    pub start_dimension: usize,
    pub dimensions: usize,

    pub kv: HashMapSet<<<V as DeletionVariant>::Iter as Iterator>::Item, Rc<V>>,
}

/// HmSearch Database using deletion variants
///
pub struct DeletionDB<V> where
    V: Value + Window + DeletionVariant,
    <<V as DeletionVariant>::Iter as Iterator>::Item: cmp::Eq + hash::Hash + clone::Clone,
{
    dimensions: usize,
    tolerance: usize,
    partition_count: usize,
    partitions: Vec<DeletionPartition<V>>,
}

/// HmSearch-indexable value
///
pub trait Value: hash::Hash + cmp::Eq + clone::Clone {
    /// Hamming distance betwen `self` and `rhs`
    ///
    fn hamming(&self, rhs: &Self) -> usize {
        self.hamming_indices(rhs).len()
    }

    /// Returns true if the hamming distance between `self` and `rhs` is less than
    /// or equal to `bound`, false otherwise
    ///
    fn hamming_lte(&self, rhs: &Self, bound: usize) -> bool {
        self.hamming(rhs) <= bound
    }

    /// Returns a vector of dimension indices whose value is different between 
    /// `self` and `rhs`
    ///
    fn hamming_indices(&self, rhs: &Self) -> Vec<usize>;
}

pub trait Window {
    // NOTE: Consider using an associated type for the output, so things like Hashed<T>
    // can return a different type

    /// Subsample on a set of dimensions
    ///
    /// `start_dimension` the index of the 1st dimension to include in the slice, 
    ///      0-indexed from least significant
    /// `dimensions` the total number of dimensions to include
    ///
    fn window(&self, start_dimension: usize, dimensions: usize) -> Self;
}

/// Return a set of single-dimensional permutation variants
///
pub trait SubstitutionVariant {
    type Iter: Iterator<Item = Self>;

    /// Substitution variants
    ///
    /// Returns an array of all possible single-column permutation of `self`.
    /// Alternately, returns the set of values with Hamming distance `1` from 
    /// `self`
    ///
    fn substitution_variants(&self, dimensions: usize) -> <Self as SubstitutionVariant>::Iter;
}

pub trait DeletionVariant {
    type Iter: Iterator;

    /// Returns an array of all possible deletion variants of `self`
    ///
    /// A "deletion variant" as defined in
    /// [Zhang](http://www.cse.unsw.edu.au/~weiw/files/SSDBM13-HmSearch-Final.pdf)
    /// is a value obtained by substituting a "deletion marker" for a single 
    /// dimension of a value.
    ///
    fn deletion_variants(&self, dimensions: usize) -> <Self as DeletionVariant>::Iter;
}
