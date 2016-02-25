use std::cmp::Eq;
use std::hash::Hash;
use std::collections;

use super::IDMap;

// This is sort of janky, but using a newtype causes duplicate trait 
// implementation errors for some reason
pub struct HashMap<K, V> {
    data: collections::HashMap<K, V>,
}

impl<ID, T> HashMap<ID, T> where
ID: Eq + Hash,
{
    pub fn new() -> HashMap<ID, T> {
        HashMap{data: collections::HashMap::new()}
    }

    pub fn with_capacity(capacity: usize) -> HashMap<ID, T> {
        HashMap{data: collections::HashMap::with_capacity(capacity)}
    }
}

impl<ID, T> IDMap<ID, T> for HashMap<ID, T> where
ID: Sync + Send + Eq + Hash,
T: Sync + Send + Clone,
{
    fn get(&self, id: ID) -> T {
        self.data.get(&id).unwrap().clone()
    }

    fn insert(&mut self, id: ID, value: T) {
        self.data.insert(id, value);
    }

    fn remove(&mut self, id: &ID) {
        self.data.remove(id);
    }
}
