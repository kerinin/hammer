use std::clone;
use std::cmp;
use std::hash;
use std::marker::PhantomData;

use std::collections::HashSet;

use libc::c_int;
use rocksdb::{DB, Writable, Options, Direction, IteratorMode};
use rustc_serialize::{Encodable, Decodable};
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};

use super::MapSet;

/// RocksDB uses RocksDB to store a mapping from keys to sets of values
///
/// Internally, k/v paris are mapped to binary RocksDB keys.  Sets of values are
/// retrieved by scanning RocksDB keys whose prefix match the given key and 
/// reconstructing the value from the "end" of the RocksDB key
///
pub struct RocksDB<K, V> {
    key: PhantomData<K>,
    value: PhantomData<V>,
    db: DB,
}

fn prefix_compare(a: &[u8], b: &[u8]) -> c_int {
    if a.len() < b.len() {
        prefix_compare(a, &b[..a.len()])
    } else if a < b {
        1
    } else if a > b {
        -1
    } else {
        0
    }
}

impl<K, V> RocksDB<K, V> {
    pub fn new(path: &str) -> RocksDB<K, V> {
        let mut opts = Options::new();
        opts.add_comparator("prefix comparator", prefix_compare);

        let db = DB::open(&opts, path).unwrap();

        RocksDB{
            key: PhantomData,
            value: PhantomData,
            db: db,
        }
    }
}


impl<K, V> MapSet for RocksDB<K, V>
where   K: clone::Clone + cmp::Eq + hash::Hash + Encodable + Decodable,
V: clone::Clone + cmp::Eq + hash::Hash + Encodable + Decodable,
{
    type Key = K;
    type Value = V;

    fn insert(&mut self, key: K, value: V) -> bool {
        let encoded_key: Vec<u8> = encode(&(key, value), SizeLimit::Infinite).unwrap();

        match self.db.get(&encoded_key) {
            Ok(Some(_)) => return false,
            Err(e) => panic!(e),
            Ok(None) => {
                self.db.put(&encoded_key, &[]).unwrap();
                true
            },
        }
    }

    fn get(&self, key: &K) -> Option<HashSet<V>> {
        let mut out = HashSet::new();
        let encoded_key_prefix: Vec<u8> = encode(&key, SizeLimit::Infinite).unwrap();

        for (k, _) in self.db.iterator(IteratorMode::From(&encoded_key_prefix, Direction::forward)) {
            let (decoded_key, decoded_value): (K, V) = decode(&k).unwrap();
            if *key == decoded_key {
                break
            }
            out.insert(decoded_value);
        }

        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    fn remove(&mut self, key: &K, value: &V) -> bool {
        let encoded_key: Vec<u8> = encode(&(key, value), SizeLimit::Infinite).unwrap();

        match self.db.get(&encoded_key) {
            Err(e) => panic!(e),
            Ok(None) => return false,
            Ok(Some(_)) => {
                self.db.delete(&encoded_key).unwrap();
                true
            }
        }
    }
}