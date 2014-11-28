use std::clone;
use std::cmp;
use std::hash;

use std::collections::{HashMap, HashSet};
use std::collections::hashmap::{Occupied, Vacant};
//use std::sync::{RWLock};

pub struct HashMapSet<K, V> {
    //lock: RWLock,
    data: HashMap<K, HashSet<V>>,
}

impl HashMapSet<Vec<u8>, Vec<u8>> {
    pub fn new() -> HashMapSet<Vec<u8>, Vec<u8>> {
        let data: HashMap<Vec<u8>, HashSet<Vec<u8>>> = HashMap::new();
        return HashMapSet {data: data};
    }
}

impl<K: hash::Hash + cmp::Eq + clone::Clone, V: hash::Hash + cmp::Eq + clone::Clone> HashMapSet<K, V> {
    pub fn insert(&mut self, key: K, value: V) -> bool {
        match self.data.entry(key.clone()) {
            Vacant(entry) => {
                let mut set: HashSet<V> = HashSet::new();
                set.insert(value);
                entry.set(set);
                true
            },
            Occupied(mut entry) => {
                entry.get_mut().insert(value)
            },
        }
    }

    pub fn find(&self, key: &K) -> Option<&HashSet<V>> {
        return self.data.find(key);
    }

    pub fn remove(&mut self, key: K, value: V) -> bool {
        let mut delete_key = false;

        let removed = match self.data.entry(key.clone()) {
            Vacant(..) => {
                false
            },
            Occupied(mut entry) => {
                let set = entry.get_mut();
                let removed = set.remove(&value);
                if set.is_empty() { delete_key = true };
                removed
            },
        };

        if delete_key {
            self.data.remove(&key.clone());
        };

        removed
    }
}

impl<K: hash::Hash + cmp::Eq, V: hash::Hash + cmp::Eq> PartialEq for HashMapSet<K, V> {
    fn eq(&self, other: &HashMapSet<K, V>) -> bool {
        return self.data.eq(&other.data);
    }

    fn ne(&self, other: &HashMapSet<K, V>) -> bool {
        return self.data.ne(&other.data);
    }
}
