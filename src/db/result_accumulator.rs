use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry::{Occupied, Vacant};

use db::value::{Value, Hamming};

pub struct ResultAccumulator<V: Value> {
    tolerance: usize,
    query: V,
    candidates: HashMap<V, (usize, usize)>,
}

impl<V: Value + Hamming> ResultAccumulator<V> {
    pub fn new(tolerance: usize, query: V) -> ResultAccumulator<V> {
        let candidates = HashMap::new();
        return ResultAccumulator {tolerance: tolerance, query: query, candidates: candidates};
    }

    pub fn insert_zero_variant(&mut self, value: &V) {
        match self.candidates.entry(value.clone()) {
            Occupied(mut entry) => {
                let &mut (mut exact_matches, _) = entry.get_mut();
                exact_matches += 1;
            },
            Vacant(entry) => {
                entry.insert((1, 0));
            }
        }
    }

    pub fn insert_one_variant(&mut self, value: &V) {
        match self.candidates.entry(value.clone()) {
            Occupied(mut entry) => {
                let &mut (_, mut one_matches) = entry.get_mut();
                one_matches += 1;
            },
            Vacant(entry) => {
                entry.insert((0, 1));
            }
        }
    }

    pub fn found_values(&self) -> Option<HashSet<V>> {
        let mut matches: HashSet<V> = HashSet::new();

        if self.tolerance % 2 == 0 {
            for (candidate, &(exact_matches, one_matches)) in self.candidates.iter() {
                // "If k is an even number, S must have at least one exact-matching
                // partition, or two 1-matching partitions"
                if exact_matches >= 1 || one_matches >= 2 {
                    if self.query.hamming_lte(candidate, self.tolerance) {
                        matches.insert(candidate.clone());
                    }
                }
            }
        } else {
            for (candidate, &(exact_matches, one_matches)) in self.candidates.iter() {
                // "If k is an odd number, S must have at least two matching partitions
                // where at least one of the matches should be an exact match, or S
                // must have at least three 1-matching partitions"
                if (exact_matches >= 1 && (exact_matches + one_matches) >= 2) || one_matches >= 3 {
                    if self.query.hamming_lte(candidate, self.tolerance) {
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
