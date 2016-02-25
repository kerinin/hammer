use std::env;
use std::fs;
use std::path::PathBuf;
use std::marker::PhantomData;

use rocksdb::{DB, Writable, Options};
use rustc_serialize::{Encodable, Decodable};
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};
use uuid::Uuid;

use super::IDMap;

pub struct TempRocksDB<ID, T> {
    dir: PathBuf,
    db: RocksDB<ID, T>,
}

impl<ID, T> TempRocksDB<ID, T> {
    pub fn with_opts(opts: Options) -> TempRocksDB<ID, T> {
        let mut dir = env::temp_dir(); 
        dir.push(&Uuid::new_v4().to_hyphenated_string());

        TempRocksDB{
            dir: dir.clone(),
            db: RocksDB::with_opts(dir.to_str().unwrap(), opts),
        }
    }

    pub fn new() -> TempRocksDB<ID, T> {
        let mut dir = env::temp_dir(); 
        dir.push(&Uuid::new_v4().to_hyphenated_string());

        TempRocksDB{
            dir: dir.clone(),
            db: RocksDB::new(dir.to_str().unwrap()),
        }
    }
}

impl<ID, T> Drop for TempRocksDB<ID, T> {
    fn drop(&mut self) {
        // Nothing we can do about it here, so ¯\_(ツ)_/¯
        let _ = fs::remove_dir_all(self.dir.to_str().unwrap());
    }
}

impl<ID, T> IDMap<ID, T> for TempRocksDB<ID, T> where
ID: Sync + Send + Encodable + Decodable,
T: Sync + Send + Encodable + Decodable,
{
    fn get(&self, id: ID) -> T {
        self.db.get(id)
    }

    fn insert(&mut self, id: ID, value: T) {
        self.db.insert(id, value)
    }

    fn remove(&mut self, id: &ID) {
        self.db.remove(id)
    }
}

pub struct RocksDB<ID, T> {
    id: PhantomData<ID>,
    value: PhantomData<T>,
    db: DB,
}

impl<ID, T> RocksDB<ID, T> {
    pub fn with_opts(path: &str, opts: Options) -> RocksDB<ID, T> {
        let db = DB::open(&opts, path).unwrap();

        RocksDB{
            id: PhantomData,
            value: PhantomData,
            db: db,
        }
    }

    pub fn new(path: &str) -> RocksDB<ID, T> {
        let db = DB::open_default(path).unwrap();

        RocksDB{
            id: PhantomData,
            value: PhantomData,
            db: db,
        }
    }
}

impl<ID, T> IDMap<ID, T> for RocksDB<ID, T> where
ID: Sync + Send + Encodable + Decodable,
T: Sync + Send + Encodable + Decodable,
{
    fn get(&self, id: ID) -> T {
        let encoded_id: Vec<u8> = encode(&id, SizeLimit::Infinite).unwrap();

        let encoded_value = self.db.get(&encoded_id).unwrap().unwrap();

        decode(&encoded_value).unwrap()
    }

    fn insert(&mut self, id: ID, value: T) {
        let encoded_id: Vec<u8> = encode(&id, SizeLimit::Infinite).unwrap();
        let encoded_value: Vec<u8> = encode(&value, SizeLimit::Infinite).unwrap();

        self.db.put(&encoded_id, &encoded_value).unwrap();
    }

    fn remove(&mut self, id: &ID) {
        let encoded_id: Vec<u8> = encode(&id, SizeLimit::Infinite).unwrap();

        self.db.delete(&encoded_id).unwrap();
    }
}
