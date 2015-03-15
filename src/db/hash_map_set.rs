use std::clone;
use std::cmp;
use std::hash;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::{Vacant, Occupied};

#[derive(Debug)]
pub struct HashMapSet<K: cmp::Eq + hash::Hash, V: cmp::Eq + hash::Hash> {
    data: HashMap<K, HashSet<V>>,
}

impl<K: hash::Hash + cmp::Eq + clone::Clone, V: hash::Hash + cmp::Eq + clone::Clone> HashMapSet<K, V> {
    pub fn new() -> HashMapSet<K, V> {
        let data: HashMap<K, HashSet<V>> = HashMap::new();
        return HashMapSet {data: data};
    }

    pub fn insert(&mut self, key: K, value: V) -> bool {
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

    pub fn get(&self, key: &K) -> Option<&HashSet<V>> {
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

//impl<K: hash::Hash + cmp::Eq, V: hash::Hash + cmp::Eq> PartialEq for HashMapSet<K, V> {
//    fn eq(&self, other: &HashMapSet<K, V>) -> bool {
//        return self.data.eq(&other.data);
//    }
//
//    fn ne(&self, other: &HashMapSet<K, V>) -> bool {
//        return self.data.ne(&other.data);
//    }
//}
