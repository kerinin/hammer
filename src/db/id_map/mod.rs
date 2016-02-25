mod echo;
mod hash_map;
mod rocks_db;

use std::ops::{Deref, DerefMut};

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

