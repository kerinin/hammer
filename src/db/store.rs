use std::cmp;

use std::collections::HashSet;

pub trait Store<K, V> {
    fn insert(&mut self, key: K, value: V) -> bool;
    fn get(&mut self, key: &K) -> Option<&HashSet<V>>;
    fn remove(&mut self, key: K, value: V) -> bool;
}
