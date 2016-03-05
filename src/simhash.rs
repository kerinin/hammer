use std::marker::PhantomData;
use std::hash::{Hasher, SipHasher};

use murmurhash3::{murmurhash3_x64_128};
use murmurhash3::{murmurhash3_x86_32};

use bit_matrix::BitTranspose;

/// SimHash hashes bytes to produce simhashes of type T
pub trait SimHash<T> {
    fn finish(&mut self) -> T;
    fn write(&mut self, bytes: &[u8]);
}

pub struct Murmur32 {
    count: usize,
    counters: Vec<usize>,
    hashes: Vec<u32>,
}

impl Murmur32 {
    pub fn new() -> Murmur32 {
        Murmur32{
            count: 0,
            counters: Vec::with_capacity(32),
            hashes: Vec::with_capacity(32),
        }
    }

    fn merge_hashes(&mut self, finishing: bool) {
        match (self.hashes.len(), finishing) {
            (0, _) => return,
            (32, _) => {},
            (_, false) => return,
            (_, true) => {
                while self.hashes.len() < 32 {
                    self.hashes.push(0);
                }
            },
        }

        self.hashes.bit_transpose_assign();

        for (i, n) in self.hashes.iter().map(|n| n.count_ones()).enumerate() {
            self.counters[i] += n as usize;
        }

        self.hashes.clear();
    }
}

impl SimHash<u32> for Murmur32 {
    fn finish(&mut self) -> u32 {
        Murmur32::merge_hashes(self, true);

        let mut simhash = 0;
        let threshold = self.count / 2;

        for (i, _) in self.counters.iter().enumerate().filter(|&(_, n)| n >= &threshold) {
            simhash = simhash | (1 << i);
        }

        simhash
    }

    fn write(&mut self, bytes: &[u8]) {
        self.merge_hashes(false);

        self.count += 1;
        self.hashes.push(murmurhash3_x86_32(bytes, 0));
    }
}

pub struct SimHasher<H=SipHasher> {
    count: usize,
    counters: Vec<usize>,
    hashes: Vec<u64>,
    hasher: PhantomData<H>,
}

impl<H: Hasher> SimHasher<H> {
    pub fn new() -> SimHasher<H> {
        SimHasher{
            count: 0,
            counters: Vec::with_capacity(64),
            hashes: Vec::with_capacity(64),
            hasher: PhantomData,
        }
    }

    fn merge_hashes(&mut self, finishing: bool) {
        match (self.hashes.len(), finishing) {
            (0, _) => return,
            (64, _) => {},
            (_, false) => return,
            (_, true) => {
                while self.hashes.len() < 64 {
                    self.hashes.push(0);
                }
            },
        }

        self.hashes.bit_transpose_assign();

        for (i, n) in self.hashes.iter().map(|n| n.count_ones()).enumerate() {
            self.counters[i] += n as usize;
        }

        self.hashes.clear();
    }
}

impl<H: Hasher + Default> SimHash<u64> for SimHasher<H> {
    fn finish(&mut self) -> u64 {
        SimHasher::merge_hashes(self, true);

        let mut simhash = 0;
        let threshold = self.count / 2;

        for (i, _) in self.counters.iter().enumerate().filter(|&(_, n)| n >= &threshold) {
            simhash = simhash | (1 << i);
        }

        simhash
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut h: H = Default::default();
        h.write(bytes);

        self.merge_hashes(false);

        self.count += 1;
        self.hashes.push(h.finish());
    }
}

pub struct Murmur128 {
    count: usize,
    counters1: Vec<usize>,
    counters2: Vec<usize>,
    hashes1: Vec<u64>,
    hashes2: Vec<u64>,
}

impl Murmur128 {
    pub fn new() -> Murmur128 {
        Murmur128{
            count: 0,
            counters1: Vec::with_capacity(64),
            counters2: Vec::with_capacity(64),
            hashes1: Vec::with_capacity(64),
            hashes2: Vec::with_capacity(64),
        }
    }

    fn merge_hashes(&mut self, finishing: bool) {
        match (self.hashes1.len(), finishing) {
            (0, _) => return,
            (64, _) => {},
            (_, false) => return,
            (_, true) => {
                while self.hashes1.len() < 64 {
                    self.hashes1.push(0);
                    self.hashes2.push(0);
                }
            },
        }

        self.hashes1.bit_transpose_assign();
        self.hashes2.bit_transpose_assign();

        for (i, n) in self.hashes1.iter().map(|n| n.count_ones()).enumerate() {
            self.counters1[i] += n as usize;
        }
        for (i, n) in self.hashes2.iter().map(|n| n.count_ones()).enumerate() {
            self.counters2[i] += n as usize;
        }

        self.hashes1.clear();
        self.hashes2.clear();
    }
}

impl SimHash<[u64; 2]> for Murmur128 {
    fn finish(&mut self) -> [u64; 2] {
        Murmur128::merge_hashes(self, true);

        let mut simhash = [0; 2];
        let threshold = self.count / 2;

        for (i, _) in self.counters1.iter().enumerate().filter(|&(_, n)| n >= &threshold) {
            simhash[0] = simhash[0] | (1 << i);
        }
        for (i, _) in self.counters2.iter().enumerate().filter(|&(_, n)| n >= &threshold) {
            simhash[1] = simhash[1] | (1 << i);
        }

        simhash
    }

    fn write(&mut self, bytes: &[u8]) {
        self.merge_hashes(false);

        self.count += 1;
        let (h1, h2) = murmurhash3_x64_128(bytes, 0);
        self.hashes1.push(h1);
        self.hashes2.push(h2);
    }
}

