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
//! Values are indexed by partitioning the value into a number of smaller pieces
//! such that the hamming distince from a partitioned query to a partitioned value 
//! must be either 0 or 1 for at least one partition.  For example, for hamming 
//! distance 3, if a value is split into 2 halves, the three 'different' dimensions 
//! will either be in the same partition, or two will be in one partition and one 
//! will be in the other.
//!
//! The number of partitions `K` which will guarantee at least one partition with a 
//! hamming distance of 1, given a query with hamming distance `D` is 
//! `K = ceil(D+3)/2`.
//!
//! This can be used to solve for an appropriate partition data type, given a
//! value type and a hamming distance:
//!
//! Value      | Hamming | Window
//! -----------+---------+--------
//! (u64, u64) | 0       | (u64, u64)
//! (u64, u64) | 1-4     | u64
//! (u64, u64) | 5-13    | u32
//! (u64, u64) | 13-29   | u16
//! (u64, u64) | 29+     | u8
//!
//! u64        | 0       | u64
//! u64        | 1-4     | u32
//! u64        | 5-13    | u16
//! u64        | 13+     | u8
//!
//! # Examples
//!
//! ```ignore
//! let dimensions = 64;
//! let tolerance = 6;
//! let mut db: SubstitutionDB<u16, u64> = SubstitutionDB::new(dimensions, tolerance)
//!
//! db.insert(0);
//! db.insert(1);
//! db.insert(3);
//! db.insert(7);
//! db.insert(1209384029384);
//!
//! let results = db.get(&0).iter().collect();
//! assert_eq!(results, vec![0,1,3,7]);
//! ```
//!

pub mod deletion;
pub mod hamming;
pub mod hashing;
pub mod substitution;
pub mod window;
pub mod map_set;

mod result_accumulator;

// mod bench; // Uncomment to get benchmarks to run

use std::collections::HashSet;

/// Abstract interface for Hamming distance databases
///
pub trait Database<'a> {
    type Value;

    fn get(&'a self, key: &Self::Value) -> Option<HashSet<Self::Value>>;
    fn insert(&mut self, key: Self::Value) -> bool;
    fn remove(&mut self, key: &Self::Value) -> bool;
}
