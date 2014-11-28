use std::fmt;
use std::cmp;
use std::hash;

use std::collections::{HashSet};

use super::value;
use super::hash_map_set::HashMapSet;
use super::find_result::{FindResult, ZeroVariant, OneVariant};

pub struct Partition<T> {
    shift: uint,
    mask: uint,

    zero_kv: HashMapSet<T, T>,
    one_kv: HashMapSet<T, T>,
}

impl<T> fmt::Show for Partition<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
        write!(f, "({:u},{:u})", self.shift, self.mask)
    }
}

impl<T: cmp::Eq + hash::Hash + PartialEq> PartialEq for Partition<T> {
    fn eq(&self, other: &Partition<T>) -> bool {
        return self.shift.eq(&other.shift) &&
            self.mask.eq(&other.mask) &&
            self.zero_kv.eq(&other.zero_kv) &&
            self.one_kv.eq(&other.one_kv);
    }

    fn ne(&self, other: &Partition<T>) -> bool {
        return self.shift.ne(&other.shift) ||
            self.mask.ne(&other.mask) ||
            self.zero_kv.ne(&other.zero_kv) ||
            self.one_kv.ne(&other.one_kv);
    }
}

impl Partition<Vec<u8>> {
    pub fn new(shift: uint, mask: uint) -> Partition<Vec<u8>> {
        let zero_kv: HashMapSet<Vec<u8>, Vec<u8>> = HashMapSet::new();
        let one_kv: HashMapSet<Vec<u8>, Vec<u8>> = HashMapSet::new();
        return Partition {shift: shift, mask: mask, zero_kv: zero_kv, one_kv: one_kv};
    }
}

impl<T: value::Value> Partition<T> {
    pub fn find(&self, key: T) -> HashSet<FindResult<T>> {
        let mut found_keys: HashSet<FindResult<T>> = HashSet::new();

        let transformed_key = key.clone().transform(self.shift, self.mask);
        match self.zero_kv.find(&transformed_key) {
            Some(keys) => {
                for k in keys.iter() {
                    println!("Did find 0:{}", transformed_key.clone());

                    found_keys.insert(ZeroVariant(k.clone()));
                }
            },
            None => {
                println!("Didn't find 0:{}", transformed_key.clone());
            },
        }

        match self.one_kv.find(&transformed_key) {
            Some(keys) => {
                for k in keys.iter() {
                    println!("Did find 1:{}", transformed_key.clone());

                    found_keys.insert(OneVariant(k.clone()));
                }
            },
            None => {
                println!("Didn't find 1:{}", transformed_key.clone());
            },
        }

        println!("Found {} keys for query {}", found_keys, key.clone());
        found_keys
    }

    pub fn insert(&mut self, key: T) -> bool {
        let transformed_key = key.clone().transform(self.shift, self.mask);

        if self.zero_kv.insert(transformed_key.clone(), key.clone()) {
            println!("Inserted {} into 0:{}", transformed_key.clone(), self);

            for k in transformed_key.permutations(self.mask).iter() {
                self.one_kv.insert(k.clone(), key.clone());

                println!("Inserted {} into 1:{}", k.clone(), self);
            }
            return true;
        } else {
            println!("Didn't insert {} into 0:{}", transformed_key.clone(), self);
        }
        return false;
    }

    pub fn remove(&mut self, key: T) -> bool {
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
    use super::super::find_result::{ZeroVariant, OneVariant};

    #[test]
    fn find_missing_key() {
        let partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];
        let keys = partition.find(a);

        assert!(keys.is_empty());
    }

    #[test]
    fn first_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];
        let b = vec![0b00000011u8];
        let mut expected = HashSet::new();
        expected.insert(ZeroVariant(a.clone()));

        assert!(partition.insert(a.clone()));

        partition.insert(b.clone());
        let keys = partition.find(a.clone());

        assert_eq!(expected, keys);
    }

    #[test]
    fn second_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];
        let mut expected = HashSet::new();
        expected.insert(ZeroVariant(a.clone()));
        partition.insert(a.clone());

        assert!(!partition.insert(a.clone()));

        let keys = partition.find(a.clone());

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
        expected.insert(OneVariant(a.clone()));
        expected.insert(ZeroVariant(b.clone()));
        expected.insert(OneVariant(c.clone()));

        partition.insert(a.clone());
        partition.insert(b.clone());
        partition.insert(c.clone());
        partition.insert(d.clone());

        let keys = partition.find(b.clone());

        assert_eq!(expected, keys);
    }

    #[test]
    fn remove_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];

        partition.insert(a.clone());

        assert!(partition.remove(a.clone()));

        let keys = partition.find(a.clone());

        assert!(keys.is_empty());
    }

    #[test]
    fn remove_missing_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];

        assert!(!partition.remove(a));
    }
}
