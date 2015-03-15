use std::fmt;

use std::collections::HashSet;

use db::hash_map_set::HashMapSet;
use db::find_result::FindResult;
use db::value::{Value, Window, SubstitutionVariant, Hamming};

pub struct Partition<V: Value> {
    start_dimension: usize,
    dimensions: usize,

    zero_kv: HashMapSet<V, V>,
    one_kv: HashMapSet<V, V>,
}

impl<V: Value> fmt::Debug for Partition<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "<Partition s{} m{}>", self.start_dimension, self.dimensions)
    }
}

impl<V: Value> PartialEq for Partition<V> {
    fn eq(&self, other: &Partition<V>) -> bool {
        return self.start_dimension.eq(&other.start_dimension) &&
            self.dimensions.eq(&other.dimensions); // &&
            //self.zero_kv.eq(&other.zero_kv) &&
            //self.one_kv.eq(&other.one_kv);
    }

    fn ne(&self, other: &Partition<V>) -> bool {
        return self.start_dimension.ne(&other.start_dimension) ||
            self.dimensions.ne(&other.dimensions); // ||
            //self.zero_kv.ne(&other.zero_kv) ||
            //self.one_kv.ne(&other.one_kv);
    }
}

impl<V: Value + Window + SubstitutionVariant + Hamming> Partition<V> {
    pub fn new(start_dimension: usize, dimensions: usize) -> Partition<V> {
        let zero_kv: HashMapSet<V, V> = HashMapSet::new();
        let one_kv: HashMapSet<V, V> = HashMapSet::new();
        return Partition {start_dimension: start_dimension, dimensions: dimensions, zero_kv: zero_kv, one_kv: one_kv};
    }

    pub fn get(&self, key: &V) -> HashSet<FindResult<V>> {
        let mut found_keys: HashSet<FindResult<V>> = HashSet::new();

        let transformed_key = &key.window(self.start_dimension, self.dimensions);
        match self.zero_kv.get(transformed_key) {
            Some(keys) => {
                for k in keys.iter() {
                    found_keys.insert(FindResult::ZeroVariant(k.clone()));
                }
            },
            None => {},
        }

        match self.one_kv.get(transformed_key) {
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
        let transformed_key = key.window(self.start_dimension, self.dimensions);

        if self.zero_kv.insert(transformed_key.clone(), key.clone()) {
            for k in transformed_key.substitution_variants(self.dimensions).iter() {
                self.one_kv.insert(k.clone(), key.clone());
            }
            return true;
        }
        return false;
    }

    pub fn remove(&mut self, key: &V) -> bool {
        let transformed_key = &key.window(self.start_dimension, self.dimensions);

        if self.zero_kv.remove(transformed_key, key) {
            for k in transformed_key.substitution_variants(self.dimensions).iter() {
                self.one_kv.remove(k, key);
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
        let partition = Partition::new(4, 4);
        let a = 0b00001111u8;
        let keys = partition.get(&a);

        assert!(keys.is_empty());
    }

    #[test]
    fn first_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = 0b11110000u8;
        let b = 0b00110000u8;
        let mut expected = HashSet::new();
        expected.insert(FindResult::ZeroVariant(a.clone()));

        assert!(partition.insert(a.clone()));

        partition.insert(b.clone());
        let keys = partition.get(&a);

        assert_eq!(expected, keys);
    }

    #[test]
    fn second_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = 0b11110000u8;
        let mut expected = HashSet::new();
        expected.insert(FindResult::ZeroVariant(a.clone()));
        partition.insert(a.clone());

        assert!(!partition.insert(a.clone()));

        let keys = partition.get(&a);

        assert_eq!(expected, keys);
    }

    #[test]
    fn find_permutation_of_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = 0b11110000u8;
        let b = 0b01110000u8;
        let c = 0b00110000u8;
        let d = 0b00010000u8;
        let mut expected = HashSet::new();
        expected.insert(FindResult::OneVariant(a.clone()));
        expected.insert(FindResult::ZeroVariant(b.clone()));
        expected.insert(FindResult::OneVariant(c.clone()));

        partition.insert(a.clone());
        partition.insert(b.clone());
        partition.insert(c.clone());
        partition.insert(d.clone());

        let keys = partition.get(&b);

        assert_eq!(expected, keys);
    }

    #[test]
    fn remove_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = 0b11110000u8;

        partition.insert(a.clone());

        assert!(partition.remove(&a));

        let keys = partition.get(&a);

        assert!(keys.is_empty());
    }

    #[test]
    fn remove_missing_key() {
        let mut partition = Partition::new(4, 4);
        let a = 0b11110000u8;

        assert!(!partition.remove(&a));
    }
}
