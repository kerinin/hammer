use std::clone;
use std::cmp;
use std::hash;
use std::marker::PhantomData;

use std::collections::hash_set;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::{Vacant, Occupied};

use super::MapSet;

#[derive(Debug)]
pub struct InMemoryHash<'a, K, V: 'a>
where   K: clone::Clone + cmp::Eq + hash::Hash, 
        V: clone::Clone + cmp::Eq + hash::Hash, 
{
    lifetime: PhantomData<&'a V>,
    data: HashMap<K, HashSet<V>>,
}

impl<'a, K, V> InMemoryHash<'a, K, V>
where   K: clone::Clone + cmp::Eq + hash::Hash, 
        V: clone::Clone + cmp::Eq + hash::Hash, 
{
    pub fn new() -> InMemoryHash<'a, K, V> {
        InMemoryHash {lifetime: PhantomData, data: HashMap::new()}
    }
}

impl<'a, K, V> MapSet<'a, K, V> for InMemoryHash<'a, K, V>
where   K: clone::Clone + cmp::Eq + hash::Hash, 
        V: clone::Clone + cmp::Eq + hash::Hash, 
{
    // type Iter = hash_set::Drain<'a, V>;
    type Iter = hash_set::IntoIter<V>;

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

    fn get(&'a self, key: &K) -> Option<hash_set::IntoIter<V>> {
        match self.data.get(key) {
            Some(h) => Some(h.clone().into_iter()),
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
                Some(mut results) => quickcheck::TestResult::from_bool(results.any(|e| e == v)),
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
                Some(mut results) => quickcheck::TestResult::from_bool(!results.any(|e| e == v2)),
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
                Some(mut results) => quickcheck::TestResult::from_bool(!results.any(|e| e == v1)),
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
                Some(mut results) => quickcheck::TestResult::from_bool(results.any(|e| e == v2)),
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
