use std::env;
use std::fs;
use std::clone::Clone;
use std::cmp::Eq;
use std::hash::Hash;
use std::ops::Drop;
use std::fmt::Debug;
use std::path::PathBuf;
use std::marker::PhantomData;

use std::collections::HashSet;

use rocksdb::{DB, Writable, Options, Direction, IteratorMode, DBIterator};
use rustc_serialize::{Encodable, Decodable};
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};
use uuid::Uuid;

use super::MapSet;

pub struct TempRocksDB<'a, K, V> {
    dir: PathBuf,
    db: RocksDB<'a, K, V>,
}

impl<'a, K, V> TempRocksDB<'a, K, V> {
    pub fn with_opts(opts: Options) -> TempRocksDB<'a, K, V> {
        let mut dir = env::temp_dir(); 
        dir.push(&Uuid::new_v4().to_hyphenated_string());

        TempRocksDB{
            dir: dir.clone(),
            db: RocksDB::with_opts(dir.to_str().unwrap(), opts),
        }
    }

    pub fn new() -> TempRocksDB<'a, K, V> {
        let mut dir = env::temp_dir(); 
        dir.push(&Uuid::new_v4().to_hyphenated_string());

        TempRocksDB{
            dir: dir.clone(),
            db: RocksDB::new(dir.to_str().unwrap()),
        }
    }
}

impl<'a, K, V> Drop for TempRocksDB<'a, K, V> {
    fn drop(&mut self) {
        // Nothing we can do about it here, so ¯\_(ツ)_/¯
        let _ = fs::remove_dir_all(self.dir.to_str().unwrap());
    }
}

impl<'a, K, V> MapSet<'a, K, V> for TempRocksDB<'a, K, V>
where   K: Debug + Clone + Eq + Hash + Encodable + Decodable,
V: Debug + Clone + Eq + Hash + Encodable + Decodable,
{
    type Iter = RocksDBIterator<'a, K, V>;

    fn insert(&mut self, key: K, value: V) -> bool {
        self.db.insert(key, value)
    }

    fn get(&'a self, key: &K) -> Option<RocksDBIterator<'a, K, V>> {
        self.db.get(key)
    }

    fn remove(&mut self, key: &K, value: &V) -> bool {
        self.db.remove(key, value)
    }
}

/// RocksDB uses RocksDB to store a mapping from keys to sets of values
///
/// Internally, k/v paris are mapped to binary RocksDB keys.  Sets of values are
/// retrieved by scanning RocksDB keys whose prefix match the given key and 
/// reconstructing the value from the "end" of the RocksDB key
///
pub struct RocksDB<'a, K, V> {
    key: PhantomData<K>,
    value: PhantomData<V>,
    lifetime: PhantomData<&'a DB>,
    db: DB,
}

impl<'a, K, V> RocksDB<'a, K, V> {
    pub fn with_opts(path: &str, opts: Options) -> RocksDB<'a, K, V> {
        let db = DB::open(&opts, path).unwrap();

        RocksDB{
            key: PhantomData,
            value: PhantomData,
            lifetime: PhantomData,
            db: db,
        }
    }

    pub fn new(path: &str) -> RocksDB<'a, K, V> {
        let db = DB::open_default(path).unwrap();

        RocksDB{
            key: PhantomData,
            value: PhantomData,
            lifetime: PhantomData,
            db: db,
        }
    }
}


impl<'a, K, V> MapSet<'a, K, V> for RocksDB<'a, K, V>
where   K: Debug + Clone + Eq + Hash + Encodable + Decodable,
V: Debug + Clone + Eq + Hash + Encodable + Decodable,
{
    type Iter = RocksDBIterator<'a, K, V>;

    fn insert(&mut self, key: K, value: V) -> bool {
        let encoded_key: Vec<u8> = encode(&(key.clone(), value.clone()), SizeLimit::Infinite).unwrap();

        match self.db.get(&encoded_key) {
            Ok(Some(_)) => {
                return false
            },
            Err(e) => {
                panic!(e)
            },
            Ok(None) => {
                self.db.put(&encoded_key, &[]).unwrap();
                true
            },
        }
    }

    fn get(&'a self, key: &K) -> Option<RocksDBIterator<'a, K, V>> {
        let encoded_key_prefix: Vec<u8> = encode(&key, SizeLimit::Infinite).unwrap();

        let mut iter = RocksDBIterator{
            key: key.clone(), 
            prefix: encoded_key_prefix.clone(),
            value: PhantomData,
            it: self.db.iterator(IteratorMode::From(&encoded_key_prefix, Direction::forward)),
        };

        if iter.is_empty() {
            None
        } else {
            Some(iter)
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

pub struct RocksDBIterator<'a, K, V> {
    key: K,
    prefix: Vec<u8>,
    value: PhantomData<V>,
    it: DBIterator<'a>,
}

impl<'a, K, V> RocksDBIterator<'a, K, V> where
K: Eq + Decodable,
V: Decodable,
{
    fn is_empty(&mut self) -> bool {
        match self.it.next() {
            Some((k, _)) => {
                let (decoded_key, _): (K, V) = decode(&k).unwrap();

                if self.key != decoded_key {
                    return true
                }

                self.it.set_mode(IteratorMode::From(&self.prefix, Direction::forward));
                false
            },
            None => true,
        }
    }
}

impl<'a, K, V> Iterator for RocksDBIterator<'a, K, V> where
K: Eq + Decodable,
V: Decodable,
{
    type Item = V;

    fn next(&mut self) -> Option<V> {
        match self.it.next() {
            Some((k, _)) => {
                let (decoded_key, decoded_value): (K, V) = decode(&k).unwrap();

                if self.key == decoded_key {
                    Some(decoded_value)
                } else {
                    None
                }
            },
            None => None,
        }
    }
}

#[cfg(test)] 
mod test {
    extern crate quickcheck;

    use self::quickcheck::quickcheck;

    use db::map_set::{MapSet, TempRocksDB};

#[test]
    fn inserted_exists() {
        fn prop(k: u64, v: u64) -> quickcheck::TestResult {
            let mut db = TempRocksDB::new();
            db.insert(k.clone(), v.clone());

            let ret = match db.get(&k) {
                Some(mut results) => quickcheck::TestResult::from_bool(results.any(|e| e == v)),
                None => quickcheck::TestResult::failed(),
            };
            ret
        }
        quickcheck(prop as fn(u64, u64) -> quickcheck::TestResult);
    }

#[test]
    fn not_inserted_no_exists() {
        fn prop(k: u64, v1: u64, v2: u64) -> quickcheck::TestResult {
            if v1 == v2 {
                return quickcheck::TestResult::discard()
            }

            let mut db = TempRocksDB::new();
            db.insert(k.clone(), v1.clone());

            let ret = match db.get(&k) {
                Some(mut results) => quickcheck::TestResult::from_bool(!results.any(|e| e == v2)),
                None => quickcheck::TestResult::failed(),
            };
            ret
        }
        quickcheck(prop as fn(u64, u64, u64) -> quickcheck::TestResult);
    }

#[test]
    fn key_not_inserted_no_exists() {
        fn prop(k1: u64, k2: u64, v: u64) -> quickcheck::TestResult {
            if k1 == k2 {
                return quickcheck::TestResult::discard()
            }

            let mut db = TempRocksDB::new();
            db.insert(k1.clone(), v.clone());

            let ret = match db.get(&k2) {
                Some(_) => quickcheck::TestResult::failed(),
                None => quickcheck::TestResult::passed(),
            };
            ret
        }
        quickcheck(prop as fn(u64, u64, u64) -> quickcheck::TestResult);
    }

#[test]
    fn deleted_no_exists() {
        fn prop(k: u64, v1: u64, v2: u64) -> quickcheck::TestResult {
            if v1 == v2 {
                return quickcheck::TestResult::discard()
            }

            let mut db = TempRocksDB::new();
            db.insert(k.clone(), v1.clone());
            db.insert(k.clone(), v2.clone());
            db.remove(&k, &v1);

            let ret = match db.get(&k) {
                Some(mut results) => quickcheck::TestResult::from_bool(!results.any(|e| e == v1)),
                None => quickcheck::TestResult::failed(),
            };
            ret
        }
        quickcheck(prop as fn(u64, u64, u64) -> quickcheck::TestResult);
    }

#[test]
    fn not_deleted_exists() {
        fn prop(k: u64, v1: u64, v2: u64) -> quickcheck::TestResult {
            if v1 == v2 {
                return quickcheck::TestResult::discard()
            }

            let mut db = TempRocksDB::new();
            db.insert(k.clone(), v1.clone());
            db.insert(k.clone(), v2.clone());
            db.remove(&k, &v1);

            let ret = match db.get(&k) {
                Some(mut results) => quickcheck::TestResult::from_bool(results.any(|e| e == v2)),
                None => quickcheck::TestResult::failed(),
            };
            ret
        }
        quickcheck(prop as fn(u64, u64, u64) -> quickcheck::TestResult);
    }

#[test]
    fn key_deleted_no_exists() {
        fn prop(k1: u64, k2: u64, v1: u64, v2: u64) -> quickcheck::TestResult {
            if k1 == k2 {
                return quickcheck::TestResult::discard()
            }

            let mut db = TempRocksDB::new();
            db.insert(k1.clone(), v1.clone());
            db.insert(k2.clone(), v2.clone());
            db.remove(&k1, &v1);

            let ret = match db.get(&k1) {
                Some(_) => quickcheck::TestResult::failed(),
                None => quickcheck::TestResult::passed(),
            };
            ret
        }
        quickcheck(prop as fn(u64, u64, u64, u64) -> quickcheck::TestResult);
    }
}
