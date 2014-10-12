extern crate num;

use std::collections::{HashMap, HashSet, Map, MutableMap};
use std::iter::Repeat;
use self::num::rational::Ratio;
use super::permutable::Permutable;

struct Partition<T> {
    shift: uint,
    mask: uint,

    kv: T,
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

impl Partition<HashMap<Vec<u8>, Vec<u8>>> {
    fn new(shift: uint, mask: uint) -> Partition<HashMap<Vec<u8>, Vec<u8>>> {
        let kv: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        return Partition {shift: shift, mask: mask, kv: kv};
    }
}

impl<T: Map<Vec<u8>, Vec<u8>> + MutableMap<Vec<u8>, Vec<u8>>> Partition<T> {

    pub fn find(&self, key: Vec<u8>) -> Option<HashSet<Vec<u8>>> {
        let transformed_key = self.transform_key(key);
        let permutations = self.permute_key(transformed_key.clone());
        let mut found_keys: HashSet<Vec<u8>> = HashSet::new();

        match self.kv.find(&transformed_key) {
            Some(key) => {
                found_keys.insert(key.clone());
            },
            None => {},
        }

        for k in permutations.iter() {
            match self.kv.find(k) {
                Some(key) => {
                    found_keys.insert(key.clone());
                },
                None => {},
            }
        }

        match found_keys.len() {
            0 => return None,
            _ => return Some(found_keys),
        }
    }

    pub fn insert(&mut self, key: Vec<u8>) -> bool {
        let transformed_key = self.transform_key(key.clone());
        let permutations = self.permute_key(transformed_key.clone());

        if self.kv.insert(transformed_key, key.clone()) {
            for k in permutations.iter() {
                self.kv.insert(k.clone(), key.clone());
            }
            return true;
        } else {
            return false;
        }
    }

    pub fn remove(&mut self, key: Vec<u8>) -> bool {
        let transformed_key = self.transform_key(key.clone());
        let permutations = self.permute_key(transformed_key.clone());

        if self.kv.remove(&transformed_key) {
            for k in permutations.iter() {
                self.kv.remove(k);
            }
            return true;
        } else {
            return false;
        }
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
        let byte_count = Ratio::new(self.shift + self.mask, 8).ceil().to_integer();
        let zero: &u8 = &0;

        let mask = vec![0b10000000u8]
            .iter()
            .chain(Repeat::new(zero))
            .take(byte_count)
            .map(|i| *i)
            .collect::<Vec<u8>>();

        return range(0u, self.mask)
            .map(|i| mask.clone().shr(&i))
            .map(|mask| key.clone().bitxor(&mask))
            .collect::<Vec<Vec<u8>>>();
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashSet};
    use super::{Partition};

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

        assert_eq!(None, keys);
    }

    #[test]
    fn find_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];
        let mut b: HashSet<Vec<u8>> = HashSet::new();
        b.insert(a.clone());

        assert!(partition.insert(a.clone()));

        let keys = partition.find(a.clone());

        assert_eq!(Some(b), keys);
    }

    #[test]
    fn find_permutation_of_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];
        let b = vec![0b00000111u8];
        let mut c: HashSet<Vec<u8>> = HashSet::new();
        c.insert(a.clone());

        assert!(partition.insert(a.clone()));

        let keys = partition.find(b.clone());

        assert_eq!(Some(c), keys);
    }

    #[test]
    fn remove_inserted_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];

        partition.insert(a.clone());

        assert!(partition.remove(a.clone()));

        let keys = partition.find(a.clone());

        assert_eq!(None, keys);
    }

    #[test]
    fn remove_missing_key() {
        let mut partition = Partition::new(4, 4);
        let a = vec![0b00001111u8];

        assert!(!partition.remove(a));
    }
}
