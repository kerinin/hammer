extern crate num;

use std::collections::{HashMap, HashSet};

use super::find_result::{FindResult, ZeroVariant, OneVariant};
use super::hamming::{Hamming};

pub struct ResultAccumulator {
    tolerance: uint,
    query: Vec<u8>,
    candidates: HashMap<Vec<u8>, Vec<uint>>,
}

impl ResultAccumulator {
    pub fn new(tolerance: uint, query: Vec<u8>) -> ResultAccumulator {
        let mut candidates: HashMap<Vec<u8>, Vec<uint>> = HashMap::new();
        return ResultAccumulator {tolerance: tolerance, query: query, candidates: candidates};
    }

    pub fn merge(&mut self, other: &FindResult<Vec<u8>>) {
        // The intermediate assignment here is to work around Rust not letting me
        // both match on self.candidates and mutate it.  The match appears to
        // require an immutable borrow and the insert obviously requires a mutable 
        // one.  This may be by design, as it enforces the consistency of the
        // value of `counts`
        let (key, value) = match *other {
            ZeroVariant(ref value) => match self.candidates.find(value) {
                Some(counts) => (value.clone(), vec![counts[0] + 1, counts[1]]),
                None => (value.clone(), vec![1u, 0u]),
            },
            OneVariant(ref value) => match self.candidates.find(value) {
                Some(counts) => (value.clone(), vec![counts[0], counts[1] + 1]),
                None => (value.clone(), vec![0u, 1u]),
            },
        };

        self.candidates.insert(key, value);
    }

    pub fn found_values(&self) -> Option<HashSet<Vec<u8>>> {
        let mut matches: HashSet<Vec<u8>> = HashSet::new();

        if self.tolerance % 2 == 0 {
            for (candidate, counts) in self.candidates.iter() {
                // "If k is an even number, S must have at least one exact-matching
                // partition, or two 1-matching partitions"
                if counts[0] >= 1 || counts[1] >= 2 {
                    if self.query.hamming(candidate) <= self.tolerance {
                        matches.insert(candidate.clone());
                    }
                }
            }
        } else {
            for (candidate, counts) in self.candidates.iter() {
                // "If k is an odd number, S must have at least two matching partitions
                // where at least one of the matches should be an exact match, or S
                // must have at least three 1-matching partitions"
                if (counts[0] >= 1 && (counts[0] + counts[1]) >= 2) || counts[1] >= 3 {
                    if self.query.hamming(candidate) <= self.tolerance {
                        matches.insert(candidate.clone());
                    }
                }
            }
        }

        match matches.len() {
            0 => return None,
            _ => return Some(matches),
        }
    }
}
