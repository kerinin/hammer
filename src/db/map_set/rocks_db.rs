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

use rocksdb::{DB, Writable, Options, Direction, IteratorMode};
use rustc_serialize::{Encodable, Decodable};
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};
use uuid::Uuid;

use super::MapSet;

pub struct TempRocksDB<K, V> {
    dir: PathBuf,
    db: RocksDB<K, V>,
}

impl<K, V> TempRocksDB<K, V> {
    pub fn with_opts(opts: Options) -> TempRocksDB<K, V> {
        let mut dir = env::temp_dir(); 
        dir.push(&Uuid::new_v4().to_hyphenated_string());

        TempRocksDB{
            dir: dir.clone(),
            db: RocksDB::with_opts(dir.to_str().unwrap(), opts),
        }
    }

    pub fn new() -> TempRocksDB<K, V> {
        let mut dir = env::temp_dir(); 
        dir.push(&Uuid::new_v4().to_hyphenated_string());

        TempRocksDB{
            dir: dir.clone(),
            db: RocksDB::new(dir.to_str().unwrap()),
        }
    }
}

impl<K, V> Drop for TempRocksDB<K, V> {
    fn drop(&mut self) {
        // Nothing we can do about it here, so ¯\_(ツ)_/¯
        let _ = fs::remove_dir_all(self.dir.to_str().unwrap());
    }
}

impl<K, V> MapSet<K, V> for TempRocksDB<K, V>
where   K: Debug + Clone + Eq + Hash + Encodable + Decodable,
V: Debug + Clone + Eq + Hash + Encodable + Decodable,
{
    fn insert(&mut self, key: K, value: V) -> bool {
        self.db.insert(key, value)
    }

    fn get(&self, key: &K) -> Option<HashSet<V>> {
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
pub struct RocksDB<K, V> {
    key: PhantomData<K>,
    value: PhantomData<V>,
    db: DB,
}

impl<K, V> RocksDB<K, V> {
    pub fn with_opts(path: &str, opts: Options) -> RocksDB<K, V> {
        let db = DB::open(&opts, path).unwrap();

        RocksDB{
            key: PhantomData,
            value: PhantomData,
            db: db,
        }
    }

    pub fn new(path: &str) -> RocksDB<K, V> {
        let db = DB::open_default(path).unwrap();

        RocksDB{
            key: PhantomData,
            value: PhantomData,
            db: db,
        }
    }
}


impl<K, V> MapSet<K, V> for RocksDB<K, V>
where   K: Debug + Clone + Eq + Hash + Encodable + Decodable,
V: Debug + Clone + Eq + Hash + Encodable + Decodable,
{
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

    fn get(&self, key: &K) -> Option<HashSet<V>> {
        let mut out = HashSet::new();
        let encoded_key_prefix: Vec<u8> = encode(&key, SizeLimit::Infinite).unwrap();

        for (k, _) in self.db.iterator(IteratorMode::From(&encoded_key_prefix, Direction::forward)) {
            let (decoded_key, decoded_value): (K, V) = decode(&k).unwrap();

            if *key != decoded_key {
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

            let mut db = TempRocksDB::new();
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

            let mut db = TempRocksDB::new();
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

            let mut db = TempRocksDB::new();
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

            let mut db = TempRocksDB::new();
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

            let mut db = TempRocksDB::new();
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
