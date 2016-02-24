use std::clone;
use std::cmp;
use std::hash;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::{Vacant, Occupied};

use super::MapSet;

#[derive(Debug)]
pub struct InMemoryHash<K, V>
where   K: clone::Clone + cmp::Eq + hash::Hash, 
        V: clone::Clone + cmp::Eq + hash::Hash, 
{
    data: HashMap<K, HashSet<V>>,
}

impl<K, V> InMemoryHash<K, V>
where   K: clone::Clone + cmp::Eq + hash::Hash, 
        V: clone::Clone + cmp::Eq + hash::Hash, 
{
    pub fn new() -> InMemoryHash<K, V> {
        InMemoryHash {data: HashMap::new()}
    }
}

impl<K, V> MapSet<K, V> for InMemoryHash<K, V>
where   K: clone::Clone + cmp::Eq + hash::Hash, 
        V: clone::Clone + cmp::Eq + hash::Hash, 
{
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

    fn get(&self, key: &K) -> Option<HashSet<V>> {
        match self.data.get(key) {
            Some(h) => Some(h.clone()),
            None => None,
        }
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
