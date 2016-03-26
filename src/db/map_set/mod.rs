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
use std::clone::Clone;
use std::cmp::Eq;
use std::hash::Hash;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

mod in_memory_hash;
mod rocks_db;

pub use self::in_memory_hash::InMemoryHash;
pub use self::rocks_db::{RocksDB, TempRocksDB};

pub trait MapSet<K, V>: Sync + Send where 
K: Clone + Eq + Hash,
V: Clone + Eq + Hash,
{

    fn insert(&mut self, key: K, value: V) -> bool;
    fn get(&self, key: &K) -> Option<HashSet<V>>;
    fn remove(&mut self, key: &K, value: &V) -> bool;
}

/*
impl<K, V, D: Deref + DerefMut> MapSet<K, V> for D where 
D: Sync + Send + Deref,
<D as Deref>::Target: MapSet<K, V>,
K: Sync + Send + Clone + Eq + Hash,
V: Sync + Send + Clone + Eq + Hash,
{
    fn insert(&mut self, key: K, value: V) -> bool {
        self.deref_mut().insert(key, value)
    }

    fn get(&self, key: &K) -> Option<HashSet<V>> {
        self.deref().get(key)
    }

    fn remove(&mut self, key: &K, value: &V) -> bool {
        self.deref_mut().remove(key, value)
    }
}
*/
