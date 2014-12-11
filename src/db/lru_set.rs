use std::clone;
use std::cmp;
use std::hash;

use std::collections::HashSet;
use std::collections::LruCache;
use std::collections::hash_map::{Occupied, Vacant};

use db::store::Store;

pub struct LruSet<K, V> {
    data: LruCache<K, HashSet<V>>,
}

impl<K: hash::Hash + cmp::Eq + clone::Clone, V: hash::Hash + cmp::Eq + clone::Clone> LruSet<K, V> {
    pub fn with_capacity(i: uint) -> LruSet<K, V> {
        let data: LruCache<K, HashSet<V>> = LruCache::new(i);
        return LruSet {data: data};
    }
}

impl<K: hash::Hash + cmp::Eq + clone::Clone, V: hash::Hash + cmp::Eq + clone::Clone> Store<K, V> for LruSet<K, V> {

    fn insert(&mut self, key: K, value: V) -> bool {
        let mut set: HashSet<V> = HashSet::new();
        set.insert(value.clone());

        match self.data.insert(key.clone(), set.clone()) {
            Some(ref mut existing) => existing.insert(value.clone()),
            None => true,
        }
    }

    fn get(&mut self, key: &K) -> Option<&HashSet<V>> {
        return self.data.get(key);
    }

    fn remove(&mut self, key: K, value: V) -> bool {

        //let mut delete_key = false;

        //let removed = match self.data.get(&key.clone()) {
        //    Some(ref mut existing) => {
        //        let removed = existing.remove(&value.clone());
        //        if existing.is_empty() { delete_key = true };
        //        removed
        //    },
        //    None => false,
        //};

        //if delete_key {
        //    self.data.remove(&key.clone());
        //};

        //removed
        true
    }
}

//impl<K: hash::Hash + cmp::PartialEq, V: hash::Hash + cmp::PartialEq> PartialEq for LruSet<K, V> {
//    fn eq(&self, other: &LruSet<K, V>) -> bool {
//        return self.data.eq(&other.data);
//    }
//
//    fn ne(&self, other: &LruSet<K, V>) -> bool {
//        //return self.data.ne(&other.data);
//        false
//    }
//}
