use std;
use std::marker::PhantomData;
use std::hash::{Hasher, SipHasher};

pub struct MinHash<H=SipHasher> {
    k: usize,
    min_hashes: Vec<u64>,
    hasher: PhantomData<H>,
}

impl<H: Hasher + Default> MinHash<H> {
    pub fn new(k: usize) -> MinHash<H> {
        let mut min_hashes = Vec::with_capacity(k);
        for _ in 0..k {
            min_hashes.push(std::u64::MAX);
        }

        MinHash{
            k: k,
            min_hashes: min_hashes,
            hasher: PhantomData,
        }
    }

    pub fn write(&mut self, bytes: &[u8]) {
        let mut h: H = Default::default();
        h.write(bytes);

        // We're going to simulate using multiple hash functions by incrementally
        // writing integers into the hash.  IE, the "first" hash function is
        // the hashed bytes plus `0`, the "second" hash function is the hashed 
        // bytes plus `0` and `1`, etc.  This should produce a sequence of 
        // values inheriting the distribution properties of the haser, and which
        // are deterministic given `bytes`
        for i in 0..self.k {
            h.write_usize(i);
            let hash = h.finish();

            if self.min_hashes[i] > hash {
                self.min_hashes[i] = hash
            }
        }
    }

    pub fn finish(&self) -> Vec<u64> {
        self.min_hashes.clone()
    }
}
