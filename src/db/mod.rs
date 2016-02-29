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
pub mod id_map;
pub mod substitution;
pub mod window;
pub mod map_set;

mod result_accumulator;

// mod bench; // Uncomment to get benchmarks to run

use std::clone::Clone;
use std::cmp::Eq;
use std::hash::Hash;
use std::path::PathBuf;
use std::collections::HashSet;
use rustc_serialize::{Encodable, Decodable};

use num::rational::Ratio;

/// Abstract interface for Hamming distance databases
///
pub trait Database<T>: Sync + Send {
    fn get(&self, key: &T) -> Option<HashSet<T>>;
    fn insert(&mut self, key: T) -> bool;
    fn remove(&mut self, key: &T) -> bool;
}

pub enum StorageBackend {
    InMemory,
    TempRocksDB,
    RocksDB(String),
}

pub trait BinaryFactory {
    fn build(tolerance: usize, storage: StorageBackend) -> Box<Database<Self>>;
}
pub trait VectorFactory {
    fn build(dimensions: usize, tolerance: usize, storage: StorageBackend) -> Box<Database<Vec<Self>>>;
}

impl VectorFactory for u8 {
    fn build(dimensions: usize, tolerance: usize, storage: StorageBackend) -> Box<Database<Vec<u8>>> {

        match storage {
            StorageBackend::InMemory => {
                let id_map = id_map::HashMap::new();
                let map_set = map_set::InMemoryHash::new();
                let db: deletion::DB<
                    Vec<u8>,
                    Vec<u8>,
                    u64,
                    u64,
                    id_map::HashMap<u64, Vec<u8>>,
                    map_set::InMemoryHash<deletion::Key<u64>, u64>,
                    > = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);

                return Box::new(db)
            },
            StorageBackend::TempRocksDB => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: deletion::DB<
                    Vec<u8>,
                    Vec<u8>,
                    u64,
                    u64,
                    id_map::TempRocksDB<u64, Vec<u8>>,
                    map_set::TempRocksDB<deletion::Key<u64>, u64>,
                    > = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);

                return Box::new(db)
            },
            StorageBackend::RocksDB(ref path) => {
                let mut id_map_path = PathBuf::from(path);
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: deletion::DB<
                    Vec<u8>,
                    Vec<u8>,
                    u64,
                    u64,
                    id_map::RocksDB<u64, Vec<u8>>,
                    map_set::RocksDB<deletion::Key<u64>, u64>,
                    > = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);

                return Box::new(db)
            },
        }
    }
}

macro_rules! echo_binary {
    ($elem:ty, $dims:expr, $([$v:ty, $b:expr]),*) => {

        impl BinaryFactory for $elem {
            fn build(tolerance: usize, storage: StorageBackend) -> Box<Database<$elem>> {

                let dimensions = $dims;
                let partitions = (tolerance + 3) / 2;
                let partition_bits = Ratio::new_raw(dimensions, partitions).ceil().to_integer();

                match (partition_bits, storage) {
                    $(
                        (b, StorageBackend::InMemory) if b <= $b => {
                            let id_map = id_map::Echo::new();
                            let map_set = map_set::InMemoryHash::new();
                            let db: substitution::DB<$elem, $v, $v, $elem, id_map::Echo<$elem>, map_set::InMemoryHash<substitution::Key<$v>, $elem>> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);

                            return Box::new(db)
                        },
                        (b, StorageBackend::TempRocksDB) if b <= $b => {
                            let id_map = id_map::Echo::new();
                            let map_set = map_set::TempRocksDB::new();
                            let db: substitution::DB<$elem, $v, $v, $elem, id_map::Echo<$elem>, map_set::TempRocksDB<substitution::Key<$v>, $elem>> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);

                            return Box::new(db)
                        },
                        (b, StorageBackend::RocksDB(ref path)) if b <= $b => {
                            let id_map = id_map::Echo::new();
                            let map_set = map_set::RocksDB::new(&path);
                            let db: substitution::DB<$elem, $v, $v, $elem, id_map::Echo<$elem>, map_set::RocksDB<substitution::Key<$v>, $elem>> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);

                            return Box::new(db)
                        },
                        )*
                            _ => panic!("Unsuported tolerance"),
                }
            }
        }
    }
}
echo_binary!(u64, 64, [u8, 8], [u16, 16], [u32, 32], [u64, 64]);
echo_binary!(u32, 32, [u8, 8], [u16, 16], [u32, 32]);
echo_binary!(u16, 16, [u8, 8], [u16, 16]);
echo_binary!(u8, 8, [u8, 8]);

macro_rules! map_binary {
    ($elem:ty, $dims:expr, $([$v:ty, $b:expr]),*) => {

        impl BinaryFactory for $elem {
            fn build(tolerance: usize, storage: StorageBackend) -> Box<Database<$elem>> {

                let dimensions = $dims;
                let partitions = (tolerance + 3) / 2;
                let partition_bits = Ratio::new_raw(dimensions, partitions).ceil().to_integer();

                match (partition_bits, storage) {
                    $(
                        (b, StorageBackend::InMemory) if b <= $b => {
                            let id_map = id_map::HashMap::new();
                            let map_set = map_set::InMemoryHash::new();
                            let db: substitution::DB<$elem, $v, $v, u64, id_map::HashMap<u64, $elem>, map_set::InMemoryHash<substitution::Key<$v>, u64>> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);

                            return Box::new(db)
                        },
                        (b, StorageBackend::TempRocksDB) if b <= $b => {
                            let id_map = id_map::TempRocksDB::new();
                            let map_set = map_set::TempRocksDB::new();
                            let db: substitution::DB<$elem, $v, $v, u64, id_map::TempRocksDB<u64, $elem>, map_set::TempRocksDB<substitution::Key<$v>, u64>> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);

                            return Box::new(db)
                        },
                        (b, StorageBackend::RocksDB(ref path)) if b <= $b => {
                            let mut id_map_path = PathBuf::from(path);
                            id_map_path.push("id_map");
                            let mut map_set_path = PathBuf::from(path);
                            map_set_path.push("map_set");

                            let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                            let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                            let db: substitution::DB<$elem, $v, $v, u64, id_map::RocksDB<u64, $elem>, map_set::RocksDB<substitution::Key<$v>, u64>> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);

                            return Box::new(db)
                        },
                        )*
                            _ => panic!("Unsuported tolerance"),
                }
            }
        }
    }
}
map_binary!([u64; 4], 256, [u8, 8], [u16, 16], [u32, 32], [u64, 64], [[u64; 2], 128], [[u64; 4], 256]);
map_binary!([u64; 2], 128, [u8, 8], [u16, 16], [u32, 32], [u64, 64], [[u64; 2], 128]);

