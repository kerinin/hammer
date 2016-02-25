use std::marker::PhantomData;

use super::{IDMap};

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
