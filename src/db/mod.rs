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

use std::path::PathBuf;
use std::mem::size_of;
use std::collections::HashSet;
use std::hash::Hash;

use num::rational::Ratio;

use db::hamming::Hamming;
use db::window::{Window, Windowable};
use db::id_map::{ToID, IDMap, Echo};
use db::map_set::{MapSet, InMemoryHash};
use db::substitution::{Key, SubstitutionVariant};

pub trait TypeMap {
    /// The data type being indexed
    type Input: Sync + Send + Clone + Eq + Hash + Hamming + Windowable<Self::Window> + ToID<Self::Identifier>;

    /// The type of windows over Input.  Window types must be large
    /// enough to store dimensions/tolerance  dimensions of Input (ideally not larger)
    type Window: Sync + Send + Clone + Eq + Hash;

    /// The type of variants computed over windows
    type Variant: Sync + Send + Clone + Eq + Hash;

    /// Value identifier - balances memory use with collision probability given
    /// the cardinality of the data being indexed
    type Identifier: Sync + Send + Clone + Eq + Hash;

    /// The value sture - maps Identifier -> Input
    type ValueStore: IDMap<Self::Identifier, Self::Input>;

    /// The variant store - maps Variant -> Identifier
    type VariantStore: Sync + Send;
}

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

pub trait BinaryDB {
    fn new(tolerance: usize, storage: StorageBackend) -> Box<Database<Self>>;
}
pub trait VectorDB {
    fn new(dimensions: usize, tolerance: usize, storage: StorageBackend) -> Box<Database<Self>>;
}

macro_rules! vector_db {
    ($elem:ty) => {
        impl VectorDB for $elem {
            fn new(dimensions: usize, tolerance: usize, storage: StorageBackend) -> Box<Database<$elem>> {

                match storage {
                    StorageBackend::InMemory => {
                        let id_map = id_map::HashMap::new();
                        let map_set = map_set::InMemoryHash::new();
                        let db: deletion::DB<
                            $elem,
                            $elem,
                            u64,
                            deletion::Dvec,
                            id_map::HashMap<u64, $elem>,
                            map_set::InMemoryHash<deletion::Key<deletion::Dvec>, u64>,
                            > = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);

                        return Box::new(db)
                    },
                    StorageBackend::TempRocksDB => {
                        let id_map = id_map::TempRocksDB::new();
                        let map_set = map_set::TempRocksDB::new();
                        let db: deletion::DB<
                            $elem,
                            $elem,
                            u64,
                            deletion::Dvec,
                            id_map::TempRocksDB<u64, $elem>,
                            map_set::TempRocksDB<deletion::Key<deletion::Dvec>, u64>,
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
                            $elem,
                            $elem,
                            u64,
                            deletion::Dvec,
                            id_map::RocksDB<u64, $elem>,
                            map_set::RocksDB<deletion::Key<deletion::Dvec>, u64>,
                            > = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);

                        return Box::new(db)
                    },
                }
            }
        }
    }
}
vector_db!(Vec<u8>);
vector_db!(Vec<u16>);
vector_db!(Vec<u32>);
vector_db!(Vec<u64>);
vector_db!(Vec<[u64; 2]>);
vector_db!(Vec<[u64; 4]>);

macro_rules! echo_binary {
    ($elem:ty, $($v:ty),*) => {

        $(
        impl TypeMap for ($elem, id_map::Echo<$elem>, map_set::InMemoryHash<substitution::Key<$v>, $elem>) {
            type Input = $elem;
            type Window = $v;
            type Variant = $v;
            type Identifier = $elem;
            type ValueStore = id_map::Echo<$elem>;
            type VariantStore = map_set::InMemoryHash<substitution::Key<$v>, $elem>;
        }

        impl TypeMap for ($elem, id_map::Echo<$elem>, map_set::TempRocksDB<substitution::Key<$v>, $elem>) {
            type Input = $elem;
            type Window = $v;
            type Variant = $v;
            type Identifier = $elem;
            type ValueStore = id_map::Echo<$elem>;
            type VariantStore = map_set::TempRocksDB<substitution::Key<$v>, $elem>;
        }

        impl TypeMap for ($elem, id_map::Echo<$elem>, map_set::RocksDB<substitution::Key<$v>, $elem>) {
            type Input = $elem;
            type Window = $v;
            type Variant = $v;
            type Identifier = $elem;
            type ValueStore = id_map::Echo<$elem>;
            type VariantStore = map_set::RocksDB<substitution::Key<$v>, $elem>;
        }
        )*

        impl BinaryDB for $elem {
            fn new(tolerance: usize, storage: StorageBackend) -> Box<Database<$elem>> {

                let dimensions = 8 * size_of::<$elem>();
                let partitions = (tolerance + 3) / 2;
                let partition_bits = Ratio::new_raw(dimensions, partitions).ceil().to_integer();

                match (partition_bits, storage) {
                    $(
                        (b, StorageBackend::InMemory) if b <= (8 * size_of::<$v>()) => {
                            let id_map = id_map::Echo::new();
                            let map_set = map_set::InMemoryHash::new();
                            let db: substitution::DB<
                                ($elem, id_map::Echo<$elem>, map_set::InMemoryHash<substitution::Key<$v>, $elem>)
                                > = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);

                            return Box::new(db)
                        },
                        (b, StorageBackend::TempRocksDB) if b <= (8 * size_of::<$v>()) => {
                            let id_map = id_map::Echo::new();
                            let map_set = map_set::TempRocksDB::new();
                            let db: substitution::DB<
                                ($elem, id_map::Echo<$elem>, map_set::TempRocksDB<substitution::Key<$v>, $elem>)
                                > = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);

                            return Box::new(db)
                        },
                        (b, StorageBackend::RocksDB(ref path)) if b <= (8 * size_of::<$v>()) => {
                            let id_map = id_map::Echo::new();
                            let map_set = map_set::RocksDB::new(&path);
                            let db: substitution::DB<
                                ($elem, id_map::Echo<$elem>, map_set::RocksDB<substitution::Key<$v>, $elem>)
                                > = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);

                            return Box::new(db)
                        },
                        )*
                            _ => panic!("Unsuported tolerance"),
                }
            }
        }
    }
}
echo_binary!(u64, u8, u16, u32, u64);
// echo_binary!(u64, u8, u16, [u8; 3], u32, [u8; 5], [u8; 6], [u8; 7], u64);
echo_binary!(u32, u8, u16, u32);
echo_binary!(u16, u8, u16);
echo_binary!(u8, u8);

macro_rules! map_binary {
    ($elem:ty, $($v:ty),*) => {

        $(
        impl TypeMap for ($elem, id_map::HashMap<u64, $elem>, map_set::InMemoryHash<substitution::Key<$v>, u64>) {
            type Input = $elem;
            type Window = $v;
            type Variant = $v;
            type Identifier = u64;
            type ValueStore = id_map::HashMap<u64, $elem>;
            type VariantStore = map_set::InMemoryHash<substitution::Key<$v>, u64>;
        }

        impl TypeMap for ($elem, id_map::TempRocksDB<u64, $elem>, map_set::TempRocksDB<substitution::Key<$v>, u64>) {
            type Input = $elem;
            type Window = $v;
            type Variant = $v;
            type Identifier = u64;
            type ValueStore = id_map::TempRocksDB<u64, $elem>;
            type VariantStore = map_set::TempRocksDB<substitution::Key<$v>, u64>;
        }

        impl TypeMap for ($elem, id_map::RocksDB<u64, $elem>, map_set::RocksDB<substitution::Key<$v>, u64>) {
            type Input = $elem;
            type Window = $v;
            type Variant = $v;
            type Identifier = u64;
            type ValueStore = id_map::RocksDB<u64, $elem>;
            type VariantStore = map_set::RocksDB<substitution::Key<$v>, u64>;
        }
        )*

        impl BinaryDB for $elem {
            fn new(tolerance: usize, storage: StorageBackend) -> Box<Database<$elem>> {

                let dimensions = 8 * size_of::<$elem>();
                let partitions = (tolerance + 3) / 2;
                let partition_bits = Ratio::new_raw(dimensions, partitions).ceil().to_integer();

                match (partition_bits, storage) {
                    $(
                        (b, StorageBackend::InMemory) if b <= (8 * size_of::<$v>()) => {
                            let id_map = id_map::HashMap::new();
                            let map_set = map_set::InMemoryHash::new();
                            let db: substitution::DB<
                                ($elem, id_map::HashMap<u64, $elem>, map_set::InMemoryHash<substitution::Key<$v>, u64>)
                                > = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);

                            return Box::new(db)
                        },
                        (b, StorageBackend::TempRocksDB) if b <= (8 * size_of::<$v>()) => {
                            let id_map = id_map::TempRocksDB::new();
                            let map_set = map_set::TempRocksDB::new();
                            let db: substitution::DB<
                                ($elem, id_map::TempRocksDB<u64, $elem>, map_set::TempRocksDB<substitution::Key<$v>, u64>)
                                > = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);

                            return Box::new(db)
                        },
                        (b, StorageBackend::RocksDB(ref path)) if b <= (8 * size_of::<$v>()) => {
                            let mut id_map_path = PathBuf::from(path);
                            id_map_path.push("id_map");
                            let mut map_set_path = PathBuf::from(path);
                            map_set_path.push("map_set");

                            let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                            let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                            let db: substitution::DB<
                                ($elem, id_map::RocksDB<u64, $elem>, map_set::RocksDB<substitution::Key<$v>, u64>)
                                > = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);

                            return Box::new(db)
                        },
                        )*
                            _ => panic!("Unsuported tolerance"),
                }
            }
        }
    }
}
map_binary!([u64; 4], u8, u16, u32, u64, [u64; 2], [u64; 4]);
map_binary!([u64; 2], u8, u16, u32, u64, [u64; 2]);

