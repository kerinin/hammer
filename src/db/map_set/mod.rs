//! Map from values to sets
//!
//! `HashMap<K,V>` maps values of type `K` to values of type `V`.  `HashSet<V>`
//! describes a set of values of type `V`.  `MapSet` is the combination of
//! these two, providing insertion & removal methods with the same semantics as
//! `HashSet` but keyed like `HashMap`.  
//!
//! # Examples
//!
//! ```ignore
//! let mut sets: InMemoryHash<&'static str, &'static str> = InMemoryHash::new();
//!
//! // Returns true if "value" has not been inserted into the set at "key"
//! assert_eq!(sets.insert("key", "value"), true);
//!
//! // ...and false if it's already in the set
//! assert_eq!(sets.insert("key", "value"), false);
//!
//! // Returns true if "value" exists in the set at "key"
//! assert_eq!(sets.remove("key", "value"), true);
//!
//! // ...and false otherwise
//! assert_eq!(sets.remove("key", "value"), false);
//!
//! sets.insert("key", "value");
//! assert_eq!(sets.get("key").contains("value"), true);
//! ```
use std::clone;
use std::cmp;
use std::hash;
use std::collections::HashSet;

mod in_memory_hash_map;
mod rocks_db;

pub use self::in_memory_hash_map::InMemoryHash;
pub use self::rocks_db::{RocksDB, TempRocksDB};

pub trait MapSet<K, V>: Sized where 
K: clone::Clone + cmp::Eq + hash::Hash,
V: clone::Clone + cmp::Eq + hash::Hash,
{

    fn insert(&mut self, key: K, value: V) -> bool;
    fn get(&self, key: &K) -> Option<HashSet<V>>;
    fn remove(&mut self, key: &K, value: &V) -> bool;
}
