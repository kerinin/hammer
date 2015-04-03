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

mod hash_map_set;
mod hashing;
mod value;
mod window;
mod substitution_db;
mod substitution_variant;
mod deletion_db;
mod deletion_variant;
mod result_accumulator;

mod bench; // Uncomment to get benchmarks to run

use std::hash::*;
use std::cmp::*;
use std::clone::*;
use std::rc::*;
use std::collections::HashSet;

use db::hash_map_set::HashMapSet;

/// Abstract interface for Hamming distance databases
///
pub trait Database<V> {
    fn new(dimensions: usize, tolerance: usize) -> Self;
    fn get(&self, key: &V) -> Option<HashSet<V>>;
    fn insert(&mut self, key: V) -> bool;
    fn remove(&mut self, key: &V) -> bool;
}
