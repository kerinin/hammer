use std::clone::Clone;
use std::default::Default;
use std::cmp::Eq;
use std::hash::Hash;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::{Vacant, Occupied};

use super::MapSet;

#[derive(Debug)]
pub struct InMemoryHash<K, V>
where   K: Sync + Send + Clone + Eq + Hash, 
        V: Sync + Send + Clone + Eq + Hash, 
{
    data: HashMap<K, HashSet<V>>,
}

impl<K, V> InMemoryHash<K, V>
where   K: Sync + Send + Clone + Eq + Hash, 
        V: Sync + Send + Clone + Eq + Hash, 
{
    pub fn new() -> InMemoryHash<K, V> {
        InMemoryHash {data: HashMap::new()}
    }
}

impl<K, V> Default for InMemoryHash<K, V>
where   K: Sync + Send + Clone + Eq + Hash, 
        V: Sync + Send + Clone + Eq + Hash, 
{
    fn default() -> InMemoryHash<K, V> {
        InMemoryHash::new()
    }
}

impl<K, V> MapSet<K, V> for InMemoryHash<K, V>
where   K: Sync + Send + Clone + Eq + Hash, 
        V: Sync + Send + Clone + Eq + Hash, 
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

#[cfg(test)] 
mod test {
    extern crate quickcheck;

    use self::quickcheck::quickcheck;

    use db::map_set::{MapSet, InMemoryHash};

    #[test]
    fn inserted_exists() {
        fn prop(k: u64, v: u64) -> quickcheck::TestResult {
            let mut db = InMemoryHash::new();
            db.insert(k.clone(), v.clone());

            match db.get(&k) {
                Some(results) => quickcheck::TestResult::from_bool(results.contains(&v)),
                None => quickcheck::TestResult::failed(),
            }
        }
        quickcheck(prop as fn(u64, u64) -> quickcheck::TestResult);
    }

    #[test]
    fn not_inserted_no_exists() {
        fn prop(k: u64, v1: u64, v2: u64) -> quickcheck::TestResult {
            if v1 == v2 {
                return quickcheck::TestResult::discard()
            }

            let mut db = InMemoryHash::new();
            db.insert(k.clone(), v1.clone());

            match db.get(&k) {
                Some(results) => quickcheck::TestResult::from_bool(!results.contains(&v2)),
                None => quickcheck::TestResult::failed(),
            }
        }
        quickcheck(prop as fn(u64, u64, u64) -> quickcheck::TestResult);
    }

    #[test]
    fn key_not_inserted_no_exists() {
        fn prop(k1: u64, k2: u64, v: u64) -> quickcheck::TestResult {
            if k1 == k2 {
                return quickcheck::TestResult::discard()
            }

            let mut db = InMemoryHash::new();
            db.insert(k1.clone(), v.clone());

            match db.get(&k2) {
                Some(_) => quickcheck::TestResult::failed(),
                None => quickcheck::TestResult::passed(),
            }
        }
        quickcheck(prop as fn(u64, u64, u64) -> quickcheck::TestResult);
    }

    #[test]
    fn deleted_no_exists() {
        fn prop(k: u64, v1: u64, v2: u64) -> quickcheck::TestResult {
            if v1 == v2 {
                return quickcheck::TestResult::discard()
            }

            let mut db = InMemoryHash::new();
            db.insert(k.clone(), v1.clone());
            db.insert(k.clone(), v2.clone());
            db.remove(&k, &v1);

            match db.get(&k) {
                Some(results) => quickcheck::TestResult::from_bool(!results.contains(&v1)),
                None => quickcheck::TestResult::failed(),
            }
        }
        quickcheck(prop as fn(u64, u64, u64) -> quickcheck::TestResult);
    }

    #[test]
    fn not_deleted_exists() {
        fn prop(k: u64, v1: u64, v2: u64) -> quickcheck::TestResult {
            if v1 == v2 {
                return quickcheck::TestResult::discard()
            }

            let mut db = InMemoryHash::new();
            db.insert(k.clone(), v1.clone());
            db.insert(k.clone(), v2.clone());
            db.remove(&k, &v1);

            match db.get(&k) {
                Some(results) => quickcheck::TestResult::from_bool(results.contains(&v2)),
                None => quickcheck::TestResult::failed(),
            }
        }
        quickcheck(prop as fn(u64, u64, u64) -> quickcheck::TestResult);
    }

    #[test]
    fn key_deleted_no_exists() {
        fn prop(k1: u64, k2: u64, v1: u64, v2: u64) -> quickcheck::TestResult {
            if k1 == k2 {
                return quickcheck::TestResult::discard()
            }

            let mut db = InMemoryHash::new();
            db.insert(k1.clone(), v1.clone());
            db.insert(k2.clone(), v2.clone());
            db.remove(&k1, &v1);

            match db.get(&k1) {
                Some(_) => quickcheck::TestResult::failed(),
                None => quickcheck::TestResult::passed(),
            }
        }
        quickcheck(prop as fn(u64, u64, u64, u64) -> quickcheck::TestResult);
    }
}
