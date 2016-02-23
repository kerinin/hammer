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
//! let mut sets: InMemoryHashMapSet<&'static str, &'static str> = MapSet::new();
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

use std::collections::*;
use std::collections::hash_map::Entry::{Vacant, Occupied};

pub trait MapSet: Sized {
    type Key: clone::Clone + cmp::Eq + hash::Hash;
    type Value: clone::Clone + cmp::Eq + hash::Hash;

    fn new() -> Self;
    fn insert(&mut self, key: Self::Key, value: Self::Value) -> bool;
    fn get(&self, key: &Self::Key) -> Option<&HashSet<Self::Value>>;
    fn remove(&mut self, key: &Self::Key, value: &Self::Value) -> bool;
}

#[derive(Debug)]
pub struct InMemoryHashMapSet<K, V>
where   K: clone::Clone + cmp::Eq + hash::Hash, 
        V: clone::Clone + cmp::Eq + hash::Hash, 
{
    data: HashMap<K, HashSet<V>>,
}

impl<K, V> MapSet for InMemoryHashMapSet<K, V>
where   K: clone::Clone + cmp::Eq + hash::Hash, 
        V: clone::Clone + cmp::Eq + hash::Hash, 
{
    type Key = K;
    type Value = V;

    fn new() -> InMemoryHashMapSet<K, V> {
        InMemoryHashMapSet {data: HashMap::new()}
    }

    fn insert(&mut self, key: K, value: V) -> bool {
        match self.data.entry(key) {
            Vacant(entry) => {
                let mut set: HashSet<V> = HashSet::new();
                set.insert(value);
                entry.insert(set);
                true
            },
            Occupied(mut entry) => {
                entry.get_mut().insert(value)
            },
        }
    }

    fn get(&self, key: &K) -> Option<&HashSet<V>> {
        return self.data.get(key);
    }

    fn remove(&mut self, key: &K, value: &V) -> bool {
        let mut delete_key = false;

        let removed = match self.data.entry(key.clone()) {
            Vacant(..) => {
                false
            },
            Occupied(mut entry) => {
                let set = entry.get_mut();
                let removed = set.remove(value);
                if set.is_empty() { delete_key = true };
                removed
            },
        };

        if delete_key {
            self.data.remove(key);
        };

        removed
    }
}
