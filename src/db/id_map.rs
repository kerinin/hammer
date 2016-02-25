extern crate num;

use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

pub trait IDMap<ID, T>: Sync + Send {
    fn get(&self, id: ID) -> T;
    fn insert(&mut self, id: ID, value: T);
    fn remove(&mut self, id: &ID);
}

impl<T, ID, D: Sync + Send + Deref + DerefMut> IDMap<ID, T> for D where 
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

pub struct Echo<T> {
    t: PhantomData<T>,
}

impl<T> Echo<T> {
    pub fn new() -> Echo<T> {
        Echo{t: PhantomData}
    }
}

impl<T: Sync + Send> IDMap<T, T> for Echo<T> {
    fn get(&self, id: T) -> T { id }
    fn insert(&mut self, _: T, _: T) {}
    fn remove(&mut self, _: &T) {}
}
