use std::fmt;

use std::collections::{HashMap, HashSet};

use db::value::Value;
use db::hash_map_set::HashMapSet;
// use db::lru_set::LruSet;
use db::find_result::FindResult;
use db::store::Store;

pub struct Partition<S> {
    shift: u64,
    mask: u64,

    zero_kv: S,
    one_kv: S,
}

impl<S> fmt::Debug for Partition<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({},{})", self.shift, self.mask)
    }
}

impl<V: PartialEq, S: Store<V, V>> PartialEq for Partition<S> {
    fn eq(&self, other: &Partition<S>) -> bool {
        return self.shift.eq(&other.shift) &&
            self.mask.eq(&other.mask); // &&
            //self.zero_kv.eq(&other.zero_kv) &&
            //self.one_kv.eq(&other.one_kv);
    }

    fn ne(&self, other: &Partition<S>) -> bool {
        return self.shift.ne(&other.shift) ||
            self.mask.ne(&other.mask); // ||
            //self.zero_kv.ne(&other.zero_kv) ||
            //self.one_kv.ne(&other.one_kv);
    }
}

impl<V: Value> Partition<HashMapSet<V, V>> {
    pub fn new(shift: u64, mask: u64) -> Partition<HashMapSet<V, V>> {
        let zero_kv: HashMapSet<V, V> = HashMapSet::new();
        let one_kv: HashMapSet<V, V> = HashMapSet::new();
        return Partition {shift: shift, mask: mask, zero_kv: zero_kv, one_kv: one_kv};
    }
}

// impl<V: Value> Partition<LruSet<V, V>> {
//     pub fn with_capacity(shift: u64, mask: u64, capacity: u64) -> Partition<LruSet<V, V>> {
//         let zero_kv: LruSet<V, V> = LruSet::with_capacity(capacity);
//         let one_kv: LruSet<V, V> = LruSet::with_capacity(capacity);
//         return Partition {shift: shift, mask: mask, zero_kv: zero_kv, one_kv: one_kv};
//     }
// }

impl<V: Value, S: Store<V, V>> Partition<S> {
    pub fn get(&mut self, key: V) -> HashSet<FindResult<V>> {
        let mut found_keys: HashSet<FindResult<V>> = HashSet::new();

        let transformed_key = key.clone().transform(self.shift, self.mask);
        match self.zero_kv.get(&transformed_key) {
            Some(keys) => {
                for k in keys.iter() {
                    found_keys.insert(FindResult::ZeroVariant(k.clone()));
                }
            },
            None => {},
        }

        match self.one_kv.get(&transformed_key) {
            Some(keys) => {
                for k in keys.iter() {
                    found_keys.insert(FindResult::OneVariant(k.clone()));
                }
            },
            None => {},
        }

        found_keys
    }

    pub fn insert(&mut self, key: V) -> bool {
        let transformed_key = key.clone().transform(self.shift, self.mask);

        if self.zero_kv.insert(transformed_key.clone(), key.clone()) {

            for k in transformed_key.permutations(self.mask).iter() {
                self.one_kv.insert(k.clone(), key.clone());
            }
            return true;
        }
        return false;
    }

    pub fn remove(&mut self, key: V) -> bool {
        let transformed_key = key.clone().transform(self.shift, self.mask);

        if self.zero_kv.remove(transformed_key.clone(), key.clone()) {
            for k in transformed_key.permutations(self.mask).iter() {
                self.one_kv.remove(k.clone(), key.clone());
            }
            return true;
        }
        return false;
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashSet};

    use super::{Partition};
    use super::super::find_result::FindResult;

    #[test]
    fn find_missing_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];
        let keys = partition.get(a);

        assert!(keys.is_empty());
    }

    #[test]
    fn first_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];
        let b = vec![0b00000011u8];
        let mut expected = HashSet::new();
        expected.insert(FindResult::ZeroVariant(a.clone()));

        assert!(partition.insert(a.clone()));

        partition.insert(b.clone());
        let keys = partition.get(a.clone());

        assert_eq!(expected, keys);
    }

    #[test]
    fn second_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];
        let mut expected = HashSet::new();
        expected.insert(FindResult::ZeroVariant(a.clone()));
        partition.insert(a.clone());

        assert!(!partition.insert(a.clone()));

        let keys = partition.get(a.clone());

        assert_eq!(expected, keys);
    }

    #[test]
    fn find_permutation_of_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];
        let b = vec![0b00000111u8];
        let c = vec![0b00000011u8];
        let d = vec![0b00000001u8];
        let mut expected = HashSet::new();
        expected.insert(FindResult::OneVariant(a.clone()));
        expected.insert(FindResult::ZeroVariant(b.clone()));
        expected.insert(FindResult::OneVariant(c.clone()));

        partition.insert(a.clone());
        partition.insert(b.clone());
        partition.insert(c.clone());
        partition.insert(d.clone());

        let keys = partition.get(b.clone());

        assert_eq!(expected, keys);
    }

    #[test]
    fn remove_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];

        partition.insert(a.clone());

        assert!(partition.remove(a.clone()));

        let keys = partition.get(a.clone());

        assert!(keys.is_empty());
    }

    #[test]
    fn remove_missing_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];

        assert!(!partition.remove(a));
    }
}
