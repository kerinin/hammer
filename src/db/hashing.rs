use std::hash::*;
use std::default::*;
use std::ops::*;
use std::cmp::*;
use std::fmt::*;
use std::borrow::*;
use std::marker::*;

use db::hamming::*;
use db::window::*;

/// Hash memoization wrapper
///
/// `Hashed` wraps another type and computes its hash on creation.  Subsequent
/// hash evaluations will use the original hash value, reducing the time 
/// required to hash large objects.
///
pub struct Hashed<T, H = SipHasher> {
    hash_value: u64,
    value: T,
    marker: PhantomData<H>,
}

/*
impl<T, H> Hashed<T,H>
where T: Hash, H: Hasher + Default 
{
    pub fn new(value: T) -> Hashed<T, H> {
        let mut hashed = Hashed {hash_value: 0, value: value, marker: PhantomData};
        let mut hasher: H = Default::default();
        hashed.value.hash(&mut hasher);
        hashed.hash_value = hasher.finish();
        hashed
    }
}
*/

impl<T,H> Hamming for Hashed<T,H>
where T: Hamming 
{
    fn hamming_indices(&self, other: &Self) -> Vec<usize> {
        let self_value: &T = &**self;
        let other_value: &T = &**other;
        self_value.hamming_indices(other_value)
    }
}

impl<T, H, W> Windowable<W> for Hashed<T, H>
where T: Windowable<W> + Hash,
    H: Hasher + Default,
{
    fn window(&self, start_dimension: usize, dimensions: usize) -> W {
        let self_value: &T = &**self;
        self_value.window(start_dimension, dimensions)
    }
}

impl<T,H> Hash for Hashed<T,H> {
    fn hash<_H>(&self, state: &mut _H) where _H: Hasher {
        self.hash_value.hash(state);
    }
}

impl<T,H> Clone for Hashed<T,H>
where T: Clone
{
    fn clone(&self) -> Hashed<T,H> {
        Hashed {
            hash_value: self.hash_value.clone(),
            value: self.value.clone(),
            marker: PhantomData,
        }
    }
}

impl<T,H> Deref for Hashed<T,H> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        &self.value
    } 
}

impl<T,H> PartialEq for Hashed<T,H> {
    fn eq(&self, other: &Self) -> bool {
        self.hash_value == other.hash_value
    }
    fn ne(&self, other: &Self) -> bool {
        self.hash_value != other.hash_value
    }
}

impl<T,H> Eq for Hashed<T,H> {}

impl<T,H> PartialOrd for Hashed<T,H> where T: PartialOrd<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.value.partial_cmp(&other.value) }
    fn lt(&self, other: &Self) -> bool { self.value.lt(&other.value) }
    fn le(&self, other: &Self) -> bool { self.value.le(&other.value) }
    fn gt(&self, other: &Self) -> bool { self.value.gt(&other.value) }
    fn ge(&self, other: &Self) -> bool { self.value.ge(&other.value) }
}

impl<T,H> Ord for Hashed<T,H> where T: Ord {
    fn cmp(&self, other: &Self) -> Ordering { self.value.cmp(&other.value) }
}

impl<T,H> Display for Hashed<T,H> where T: Display {
    fn fmt(&self, f: &mut Formatter) -> Result { self.value.fmt(f) }
}

impl<T,H> Debug for Hashed<T,H> where T: Debug {
    fn fmt(&self, f: &mut Formatter) -> Result { self.value.fmt(f) }
}

impl<T,H> Borrow<T> for Hashed<T,H> {
    fn borrow(&self) -> &T {
        &self.value
    }
}
