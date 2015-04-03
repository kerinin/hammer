//! Map from values to sets
//!
//! `HashMap<K,V>` maps values of type `K` to values of type `V`.  `HashSet<V>`
//! describes a set of values of type `V`.  `HashMapSet` is the combination of
//! these two, providing insertion & removal methods with the same semantics as
//! `HashSet` but keyed like `HashMap`.  
//!
//! # Examples
//!
//! ```ignore
//! let mut sets = HashMapSet::new();
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
use std::marker;

use std::collections::*;
use std::collections::hash_map::Entry::{Vacant, Occupied};

#[derive(Debug)]
pub struct HashMapSet<K, V, SH = hash_map::RandomState, SS = hash_map::RandomState>
where   K: cmp::Eq + hash::Hash, 
        V: cmp::Eq + hash::Hash, 
        SH: marker::Sized + clone::Clone + hash_state::HashState,
        SS: marker::Sized + clone::Clone + hash_state::HashState,
{
    set_state: SS,
    data: HashMap<K, HashSet<V, SS>, SH>,
}

impl<K, V> HashMapSet<K, V, hash_map::RandomState, hash_map::RandomState>
where   K: cmp::Eq + hash::Hash, 
        V: cmp::Eq + hash::Hash, 
{
    pub fn new() -> HashMapSet<K, V, hash_map::RandomState, hash_map::RandomState> {
        let state = hash_map::RandomState::new();
        let data: HashMap<K, HashSet<V>> = HashMap::with_hash_state(state.clone());
        return HashMapSet {set_state: state, data: data};
    }
}

impl<K, V, SH, SS> HashMapSet<K, V, SH, SS>
where   K: clone::Clone + cmp::Eq + hash::Hash, 
        V: cmp::Eq + hash::Hash, 
        SH: clone::Clone + hash_state::HashState,
        SS: clone::Clone + hash_state::HashState,
{
    pub fn with_hash_state(hash_state: SH, set_state: SS) -> HashMapSet<K, V, SH, SS> {
        let data: HashMap<K, HashSet<V, SS>, SH> = HashMap::with_hash_state(hash_state.clone());
        return HashMapSet {set_state: set_state, data: data};
    }

    pub fn insert(&mut self, key: K, value: V) -> bool {
        match self.data.entry(key) {
            Vacant(entry) => {
                let mut set: HashSet<V, SS> = HashSet::with_hash_state(self.set_state.clone());
                set.insert(value);
                entry.insert(set);
                true
            },
            Occupied(mut entry) => {
                entry.get_mut().insert(value)
            },
        }
    }

    pub fn get(&self, key: &K) -> Option<&HashSet<V, SS>> {
        return self.data.get(key);
    }

    pub fn remove(&mut self, key: &K, value: &V) -> bool {
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
