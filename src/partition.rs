extern crate num;

use std::collections::bitv;
use std::iter::Repeat;
use std::fmt;
use std::cmp;
use std::hash;

use std::collections::{HashSet};

use super::permutable::Permutable;
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

impl<T> Partition<T> {
    fn mask_bytes(&self) -> Vec<u8> {
        let full_byte_count = self.mask / 8;
        let tail_bits = self.mask % 8;
        let partial_mask = 0b11111111u8.shl(&(8-tail_bits));

        let mut out = Repeat::new(0b11111111u8).take(full_byte_count).collect::<Vec<u8>>();
        out.push(partial_mask);

        return out;
    }
}

impl Partition<Vec<u8>> {
    pub fn new(shift: uint, mask: uint) -> Partition<Vec<u8>> {
        let zero_kv: HashMapSet<Vec<u8>, Vec<u8>> = HashMapSet::new();
        let one_kv: HashMapSet<Vec<u8>, Vec<u8>> = HashMapSet::new();
        return Partition {shift: shift, mask: mask, zero_kv: zero_kv, one_kv: one_kv};
    }

    pub fn find(&self, key: Vec<u8>) -> HashSet<FindResult<Vec<u8>>> {
        let mut found_keys: HashSet<FindResult<Vec<u8>>> = HashSet::new();

        let transformed_key = self.transform_key(key.clone());
        match self.zero_kv.find(&transformed_key) {
            Some(keys) => {
                for k in keys.iter() {
                    let s = transformed_key.clone().iter().map(|b| format!("{:08t}", *b)).collect::<Vec<String>>();
                    println!("Did find 0:{}", s);

                    found_keys.insert(ZeroVariant(k.clone()));
                }
            },
            None => {
                let s = transformed_key.clone().iter().map(|b| format!("{:08t}", *b)).collect::<Vec<String>>();
                println!("Didn't find 0:{}", s);
            },
        }

        match self.one_kv.find(&transformed_key) {
            Some(keys) => {
                for k in keys.iter() {
                    let s = transformed_key.clone().iter().map(|b| format!("{:08t}", *b)).collect::<Vec<String>>();
                    println!("Did find 1:{}", s);

                    found_keys.insert(OneVariant(k.clone()));
                }
            },
            None => {
                let s = transformed_key.clone().iter().map(|b| format!("{:08t}", *b)).collect::<Vec<String>>();
                println!("Didn't find 1:{}", s);
            },
        }

        let s = key.clone().iter().map(|b| format!("{:08t}", *b)).collect::<Vec<String>>();
        println!("Found {} keys for query {}", found_keys, s);
        found_keys
    }

    pub fn insert(&mut self, key: Vec<u8>) -> bool {
        let transformed_key = self.transform_key(key.clone());

        if self.zero_kv.insert(transformed_key.clone(), key.clone()) {
            let s = transformed_key.clone().iter().map(|b| format!("{:08t}", *b)).collect::<Vec<String>>();
            println!("Inserted {} into 0:{}", s, self);

            for k in self.permute_key(transformed_key.clone()).iter() {
                self.one_kv.insert(k.clone(), key.clone());

                let s = k.clone().iter().map(|b| format!("{:08t}", *b)).collect::<Vec<String>>();
                println!("Inserted {} into 1:{}", s, self);
            }
            return true;
        } else {
            let s = transformed_key.clone().iter().map(|b| format!("{:08t}", *b)).collect::<Vec<String>>();
            println!("Didn't insert {} into 0:{}", s, self);
        }
        return false;
    }

    pub fn remove(&mut self, key: Vec<u8>) -> bool {
        let transformed_key = self.transform_key(key.clone());

        if self.zero_kv.remove(transformed_key.clone(), key.clone()) {
            for k in self.permute_key(transformed_key.clone()).iter() {
                self.one_kv.remove(k.clone(), key.clone());
            }
            return true;
        }
        return false;
    }

    /*
     * Transform the full key into this partition's keyspace.  Generally involves
     * dropping all but a fraction of the data
     */
    fn transform_key(&self, key: Vec<u8>) -> Vec<u8> {
        let shifted = key.shl(&self.shift);

        return shifted.bitand(&self.mask_bytes());
    }

    /*
     * Returns an array containing all possible binary 1-permutations of the key
     */
    fn permute_key(&self, key: Vec<u8>) -> Vec<Vec<u8>> {
        let key_bitv = bitv::from_bytes(key.as_slice());
        //let key_str = key.iter().map(|b| format!("{:08t}", *b)).collect::<Vec<String>>();
        //println!("permuting {}", key_str);
        
        return range(0u, self.mask)
            .map(|i| -> Vec<u8> {
                let mut permutation = key_bitv.clone();
                let old_val = permutation.get(i);
                permutation.set(i, !old_val);

                //let permutation_str = permutation.to_bytes().iter().map(|b| format!("{:08t}", *b)).collect::<Vec<String>>();
                //println!("returing permutation {} -> {}", key_str, permutation_str);

                permutation.to_bytes()
            })
            .collect::<Vec<Vec<u8>>>();
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashSet};

    use super::{Partition};
    use super::super::find_result::{ZeroVariant, OneVariant};

    #[test]
    fn mask_bytes() {
        let partition = Partition::new(12, 12);

        assert_eq!(partition.mask_bytes(), vec![0b11111111u8, 0b11110000u8]);
    }

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
