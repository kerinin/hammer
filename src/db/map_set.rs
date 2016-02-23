//! Map from values to sets
//!
//! `HashMap<K,V>` maps values of type `K` to values of type `V`.  `HashSet<V>`
//! describes a set of values of type `V`.  `MapSet` is the combination of
//! these two, providing insertion & removal methods with the same semantics as
//! `HashSet` but keyed like `HashMap`.  
//!
//! # Examples
//!
//! ```ignore
//! let mut sets: InMemoryHashMapSet<&'static str, &'static str> = MapSet::new();
//!
//! // Returns true if "value" has not been inserted into the set at "key"
//! assert_eq!(sets.insert("key", "value"), true);
//!
//! // ...and false if it's already in the set
//! assert_eq!(sets.insert("key", "value"), false);
//!
//! // Returns true if "value" exists in the set at "key"
//! assert_eq!(sets.remove("key", "value"), true);
//!
//! // ...and false otherwise
//! assert_eq!(sets.remove("key", "value"), false);
//!
//! sets.insert("key", "value");
//! assert_eq!(sets.get("key").contains("value"), true);
//! ```
use std::clone;
use std::cmp;
use std::hash;
use std::marker::PhantomData;

use std::collections::*;
use std::collections::hash_map::Entry::{Vacant, Occupied};

use libc::c_int;
use rocksdb::*;
use rustc_serialize::*;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};

pub trait MapSet: Sized {
    type Key: clone::Clone + cmp::Eq + hash::Hash;
    type Value: clone::Clone + cmp::Eq + hash::Hash;

    fn new() -> Self;
    fn insert(&mut self, key: Self::Key, value: Self::Value) -> bool;
    fn get(&self, key: &Self::Key) -> Option<HashSet<Self::Value>>;
    fn remove(&mut self, key: &Self::Key, value: &Self::Value) -> bool;
}

/// RocksDBMapSet uses RocksDB to store a mapping from keys to sets of values
///
/// Internally, k/v paris are mapped to binary RocksDB keys.  Sets of values are
/// retrieved by scanning RocksDB keys whose prefix match the given key and 
/// reconstructing the value from the "end" of the RocksDB key
///
pub struct RocksDBMapSet<K, V> {
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

impl<K, V> MapSet for RocksDBMapSet<K, V>
where   K: clone::Clone + cmp::Eq + hash::Hash + Encodable + Decodable,
V: clone::Clone + cmp::Eq + hash::Hash + Encodable + Decodable,
{
    type Key = K;
    type Value = V;

    fn new() -> RocksDBMapSet<K, V> {
        let path = "path/to/rocksdb";
        let mut opts = Options::new();
        opts.add_comparator("prefix comparator", prefix_compare);

        let db = DB::open(&opts, path).unwrap();

        RocksDBMapSet{
            key: PhantomData,
            value: PhantomData,
            db: db,
        }
    }

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
            // Some(&out)
            None
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

#[derive(Debug)]
pub struct InMemoryHashMapSet<K, V>
where   K: clone::Clone + cmp::Eq + hash::Hash, 
        V: clone::Clone + cmp::Eq + hash::Hash, 
{
    data: HashMap<K, HashSet<V>>,
}

impl<K, V> MapSet for InMemoryHashMapSet<K, V>
where   K: clone::Clone + cmp::Eq + hash::Hash, 
        V: clone::Clone + cmp::Eq + hash::Hash, 
{
    type Key = K;
    type Value = V;

    fn new() -> InMemoryHashMapSet<K, V> {
        InMemoryHashMapSet {data: HashMap::new()}
    }

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
