use std::hash::*;
use std::default::*;
use std::ops::*;
use std::cmp::*;
use std::fmt::*;
use std::borrow::*;
use std::collections::hash_state::*;
use std::marker::*;

/// `Hasher` implementation for pre-hashed types
///
/// This hasher is intended to be used with types that expect to hash a value,
/// in cases where your data is already hashed (for example, using `Hashed<T>`)
///
struct ProxyHasher {
    hash: u64,
}

/// Hash memoization wrapper
///
/// `Hashed` wraps another type and computes its hash on creation.  Subsequent
/// hash evaluations will use the original hash value, reducing the time 
/// required to hash large objects.
///
#[derive(Clone)]
struct Hashed<T, H = SipHasher> {
    hash_value: u64,
    value: T,
    marker: PhantomData<H>,
}

impl<T, H> Hashed<T,H> where T: Hash, H: Hasher + Default {
    fn new(value: T) -> Hashed<T, H> {
        let mut hashed = Hashed {hash_value: 0, value: value, marker: PhantomData};
        let mut hasher: H = Default::default();
        hashed.value.hash(&mut hasher);
        hashed.hash_value = hasher.finish();
        hashed
    }

    fn new_with_state<S>(value: T, state: S) -> Hashed<T,H> where S: HashState<Hasher = H> {
        let mut hashed = Hashed {hash_value: 0, value: value, marker: PhantomData};
        let mut hasher: H = state.hasher();
        hashed.value.hash(&mut hasher);
        hashed.hash_value = hasher.finish();
        hashed
    }
}

impl<T,H> Hash for Hashed<T,H> {
    fn hash<_H>(&self, state: &mut _H) where _H: Hasher {
        state.write_u64(self.hash_value);
    }

    fn hash_slice<_H>(data: &[Self], state: &mut _H) where _H: Hasher {
        for hashed in data.iter() {
            state.write_u64(hashed.hash_value);
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

impl Default for ProxyHasher {
    fn default() -> ProxyHasher {
        ProxyHasher {hash: 0}
    }
}

impl Hasher for ProxyHasher {
    fn finish(&self) -> u64 {
        self.hash.clone()
    }
    fn write(&mut self, bytes: &[u8]) {
        println!("ProxyHasher#write called with bytes, not doing anything...");
    }
    fn write_u8(&mut self, i: u8) {
        self.hash = self.hash ^ i as u64;
    }
    fn write_u16(&mut self, i: u16) {
        self.hash = self.hash ^ i as u64;
    }
    fn write_u32(&mut self, i: u32) {
        self.hash = self.hash ^ i as u64;
    }
    fn write_u64(&mut self, i: u64) {
        self.hash = self.hash ^ i;
    }
    fn write_usize(&mut self, i: usize) {
        self.hash = self.hash ^ i as u64;
    }
    fn write_i8(&mut self, i: i8) {
        self.hash = self.hash ^ i as u64;
    }
    fn write_i16(&mut self, i: i16) {
        self.hash = self.hash ^ i as u64;
    }
    fn write_i32(&mut self, i: i32) {
        self.hash = self.hash ^ i as u64;
    }
    fn write_i64(&mut self, i: i64) {
        self.hash = self.hash ^ i as u64;
    }
    fn write_isize(&mut self, i: isize) {
        self.hash = self.hash ^ i as u64;
    }
}
