mod echo;
mod hash_map;
mod rocks_db;

use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

use fnv::FnvHasher;

pub use self::echo::Echo;
pub use self::hash_map::HashMap;
pub use self::rocks_db::{RocksDB, TempRocksDB};

pub trait IDMap<ID, T>: Sync + Send {
    fn get(&self, id: ID) -> T;
    fn insert(&mut self, id: ID, value: T);
    fn remove(&mut self, id: &ID);
}

impl<T, ID, D: Deref + DerefMut> IDMap<ID, T> for D where 
D: Sync + Send,
<D as Deref>::Target: IDMap<ID, T>,
{
    fn get(&self, id: ID) -> T {
        self.deref().get(id)
    }

    fn insert(&mut self, id: ID, value: T) { 
        self.deref_mut().insert(id, value)
    }

    fn remove(&mut self, id: &ID) {
        self.deref_mut().remove(id)
    }
}

pub trait ToID<T> {
    fn to_id(self) -> T;
}

impl<T> ToID<T> for T {
    fn to_id(self) -> T { self }
}

macro_rules! to_id_hash_fnv {
    ($elem:ty) => {
        impl ToID<u64> for $elem {
            fn to_id(self) -> u64 {
                let mut s = FnvHasher::default();
                self.hash(&mut s);
                s.finish()
            }
        }
    }
}
to_id_hash_fnv!([u64; 2]);
to_id_hash_fnv!([u64; 4]);
