extern crate num;

use std::fmt;
use std::cmp::*;
use std::clone::*;
use std::hash::*;
use std::collections::*;
use std::marker::PhantomData;

use self::num::rational::Ratio;

use db::*;
use db::hash_map_set::*;
use db::result_accumulator::*;
use db::substitution_variant::*;
use db::hamming::*;
use db::window::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key<T> {
    One(Window, T),
    Zero(Window, T),
}

/// HmSearch Database using substitution variants
///
pub struct SubstitutionDB<T, W, S = InMemoryHashMapSet<Key<W>, T>> {
    value: PhantomData<T>,
    window: PhantomData<W>,
    dimensions: usize,
    tolerance: usize,
    partition_count: usize,
    partitions: Vec<Window>,
    store: S,
}

impl<T, W> Database for SubstitutionDB<T, W> where
T: Clone + Eq + Hash + Hamming + Windowable<W>,
W: Clone + Eq + Hash + SubstitutionVariant,
{
    type Value = T;

    /// Create a new DB
    ///
    /// Partitions the keyspace as evenly as possible - all partitions
    /// will have either N or N-1 dimensions
    ///
    fn new(dimensions: usize, tolerance: usize) -> SubstitutionDB<T, W> {

        // Determine number of partitions
        let partition_count = if tolerance == 0 {
            1
        } else if tolerance > dimensions {
            (dimensions + 3) / 2
        } else {
            (tolerance + 3) / 2
        };

        // Determine how many dimensions to allocate to each partition
        let head_width = Ratio::new(dimensions, partition_count).ceil().to_integer() as usize;
        let tail_width = Ratio::new(dimensions, partition_count).floor().to_integer() as usize;
        let head_count = dimensions % partition_count;
        let tail_count = partition_count - head_count;

        // Build the partitions
        let mut partitions: Vec<Window> = Vec::with_capacity(head_count + tail_count);
        for i in 0..head_count {
            let start_dimension = i * head_width;
            let dimensions = head_width;

            partitions.push(Window{start_dimension: start_dimension, dimensions: dimensions});
        }
        for i in 0..tail_count {
            let start_dimension = (head_count * head_width) + (i * tail_width);
            let dimensions = tail_width;

            partitions.push(Window{start_dimension: start_dimension, dimensions: dimensions});
        }

        // Done!
        return SubstitutionDB {
            value: PhantomData,
            window: PhantomData,
            dimensions: dimensions,
            tolerance: tolerance,
            partition_count: partition_count,
            partitions: partitions,
            store: HashMapSet::new(),
        };
    }

    /// Get all indexed values within `self.tolerance` hammind distance of `key`
    ///
    fn get(&self, key: &T) -> Option<HashSet<T>> {
        let mut results = ResultAccumulator::new(self.tolerance, key.clone());

        // Split across tasks?
        for window in self.partitions.iter() {

            let transformed_key = &key.window(window.start_dimension, window.dimensions);

            match self.store.get(&Key::Zero(window.clone(), transformed_key.clone())) {
                Some(keys) => {
                    for k in keys.iter() {
                        results.insert_zero_variant(k)
                    }
                },
                None => {},
            }

            match self.store.get(&Key::One(window.clone(), transformed_key.clone())) {
                Some(keys) => {
                    for k in keys.iter() {
                        results.insert_one_variant(k)
                    }
                },
                None => {},
            }
        }

        results.found_values()
    }

    /// Insert `key` into indices
    ///
    /// Returns true if key was added to ANY index
    ///
    fn insert(&mut self, key: T) -> bool {

        // Split across tasks?
        self.partitions.clone().into_iter().map(|window| {

            let transformed_key = key.window(window.start_dimension, window.dimensions);

            if self.store.insert(Key::Zero(window.clone(), transformed_key.clone()), key.clone()) {
                for k in transformed_key.substitution_variants(window.dimensions) {
                    self.store.insert(Key::One(window.clone(), k.clone()), key.clone());
                }
                true
            } else {
                false
            }

            // Collecting first to force evaluation
        }).collect::<Vec<bool>>().iter().any(|i| *i)
    }

    /// Remove `key` from indices
    ///
    /// Returns true if key was removed from ANY index
    ///
    fn remove(&mut self, key: &T) -> bool {
        // Split across tasks?
        self.partitions.clone().into_iter().map(|window| {
            let transformed_key = &key.window(window.start_dimension, window.dimensions);

            if self.store.remove(&Key::Zero(window.clone(), transformed_key.clone()), key) {
                for ref k in transformed_key.substitution_variants(window.dimensions) {
                    self.store.remove(&Key::Zero(window.clone(), k.clone()), key);
                }
                true
            } else {
                false
            }

            // Collecting first to force evaluation
        }).collect::<Vec<bool>>().iter().any(|i| *i)
    }
}

impl<T, W> fmt::Debug for SubstitutionDB<T, W> where
T: Clone + Eq + Hash,
W: Clone + Eq + Hash,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}:{}:{})", self.dimensions, self.tolerance, self.partition_count)
    }
}

impl<T, W> PartialEq for SubstitutionDB<T, W> where
T: Clone + Eq + Hash,
W: Clone + Eq + Hash,
{
    fn eq(&self, other: &SubstitutionDB<T, W>) -> bool {
        return self.dimensions == other.dimensions &&
            self.tolerance == other.tolerance &&
            self.partition_count == other.partition_count;// &&
            //self.partitions.eq(&other.partitions);
    }

    fn ne(&self, other: &SubstitutionDB<T, W>) -> bool {
        return self.dimensions != other.dimensions ||
            self.tolerance != other.tolerance ||
            self.partition_count != other.partition_count; // ||
            //self.partitions.eq(&other.partitions);
    }
}

// Internal tests
#[test]
fn test_sdb_partition_evenly() {
    let a: SubstitutionDB<u64, u64> = Database::new(32, 5);
    let b = SubstitutionDB {
        value: PhantomData,
        window: PhantomData,
        dimensions: 32,
        tolerance: 5,
        partition_count: 4,
        partitions: vec![
            Window{start_dimension: 0, dimensions: 8},
            Window{start_dimension: 8, dimensions: 8},
            Window{start_dimension: 16, dimensions: 8},
            Window{start_dimension: 24, dimensions: 8},
        ],
        store: HashMapSet::new(),
    };

    assert_eq!(a, b);
}

#[test]
fn test_sdb_partition_unevenly() {
    let a: SubstitutionDB<u64, u64> = Database::new(32, 7);
    let b = SubstitutionDB {
        value: PhantomData,
        window: PhantomData,
        dimensions: 32,
        tolerance: 7,
        partition_count: 5,
        partitions: vec![
            Window{start_dimension:0, dimensions: 7},
            Window{start_dimension:7, dimensions: 7},
            Window{start_dimension:14, dimensions: 6},
            Window{start_dimension:20, dimensions: 6},
            Window{start_dimension:26, dimensions: 6}
        ],
        store: HashMapSet::new(),
    };

    assert_eq!(a, b);
}

#[test]
fn test_sdb_partition_too_many() {
    let a: SubstitutionDB<u64, u64> = Database::new(4, 8);
    let b = SubstitutionDB {
        value: PhantomData,
        window: PhantomData,
        dimensions: 4,
        tolerance: 8,
        partition_count: 3,
        partitions: vec![
            Window{start_dimension:0, dimensions: 2},
            Window{start_dimension:2, dimensions: 1},
            Window{start_dimension:3, dimensions: 1},
        ],
        store: HashMapSet::new(),
    };

    assert_eq!(a, b);
}

#[test]
fn test_sdb_partition_zero() {
    let a: SubstitutionDB<u64, u64> = Database::new(32, 0);
    let b = SubstitutionDB {
        value: PhantomData,
        window: PhantomData,
        dimensions: 32,
        tolerance: 0,
        partition_count: 1,
        partitions: vec![
            Window{start_dimension:0, dimensions: 32},
        ],
        store: HashMapSet::new(),
    };

    assert_eq!(a, b);
}

#[test]
fn test_sdb_partition_with_no_bytes() {
    let a: SubstitutionDB<u64, u64> = Database::new(0, 0);
    let b = SubstitutionDB {
        value: PhantomData,
        window: PhantomData,
        dimensions: 0,
        tolerance: 0,
        partition_count: 1,
        partitions: vec![
            Window{start_dimension:0, dimensions: 0},
        ],
        store: HashMapSet::new(),
    };

    assert_eq!(a, b);
}


#[cfg(test)]
mod test {
    extern crate rand;
    extern crate quickcheck;

    use self::quickcheck::quickcheck;

    use std::collections::HashSet;
    use self::rand::{thread_rng, sample, Rng};

    use db::*;
    use db::substitution_db::*;

    #[test]
    fn find_missing_key() {
        let p: SubstitutionDB<u64, u64> = Database::new(8, 2);
        let a = 0b11111111u64;
        let keys = p.get(&a);

        assert_eq!(None, keys);
    }

    #[test]
    fn insert_first_key() {
        let mut p: SubstitutionDB<u64, u64> = Database::new(8, 2);
        let a = 0b11111111u64;

        assert!(p.insert(a.clone()));
    }

    #[test]
    fn insert_second_key() {
        let mut p: SubstitutionDB<u64, u64> = Database::new(8, 2);
        let a = 0b11111111u64;

        p.insert(a.clone());

        assert!(!p.insert(a.clone()));
    }

    #[test]
    fn find_inserted_key() {
        let mut p: SubstitutionDB<u64, u64> = Database::new(8, 2);
        let a = 0b11111111u64;
        let mut b: HashSet<u64> = HashSet::new();
        b.insert(a.clone());

        assert!(p.insert(a.clone()));

        let keys = p.get(&a);

        assert_eq!(Some(b), keys);
    }

    #[test]
    fn find_permutations_of_inserted_key() {
        let mut p: SubstitutionDB<u64, u64> = Database::new(8, 2);
        let a = 0b00001111u64;
        let b = 0b00000111u64;
        let mut c = HashSet::new();
        c.insert(a.clone());

        p.insert(a.clone());

        let keys = p.get(&b);

        assert_eq!(Some(c), keys);
    }

    #[test]
    fn find_permutations_of_multiple_similar_keys() {
        let mut p: SubstitutionDB<u64, u64> = Database::new(8, 4);
        let a = 0b00000000u64;
        let b = 0b10000000u64;
        let c = 0b10000001u64;
        let d = 0b11000001u64;
        let e = 0b11000011u64;
        let mut f: HashSet<u64> = HashSet::new();
        f.insert(b.clone());
        f.insert(c.clone());
        f.insert(d.clone());
        f.insert(e.clone());

        p.insert(b.clone());
        p.insert(c.clone());
        p.insert(d.clone());
        p.insert(e.clone());

        let keys = p.get(&a);

        assert_eq!(Some(f), keys);
    }

    #[test]
    fn find_permutation_of_inserted_key() {
        let mut rng1 = thread_rng();
        let mut rng2 = thread_rng();
        let dimensions = 8usize;
        let max_hd = 3usize;
        let start_dimensions_seq = rng1.gen_iter::<usize>()
            .map(|i| sample(&mut rng2, 0..dimensions, i % max_hd));

        for start_dimensions in start_dimensions_seq.take(1000usize) {
            let mut p: SubstitutionDB<u64, u64> = Database::new(dimensions, max_hd);
            let a = 0b11111111u64;

            let mut b = a.clone();
            for start_dimension in start_dimensions.iter() {
                b = b ^ (0b10000000u64 >> *start_dimension);
            }

            let mut c: HashSet<u64> = HashSet::new();
            c.insert(a.clone());

            assert!(p.insert(a.clone()));

            let keys = p.get(&b);

            assert_eq!(Some(c), keys);
        }
    }

    #[test]
    fn dont_find_permutation_of_inserted_key() {
        let mut rng1 = thread_rng();
        let mut rng2 = thread_rng();
        let dimensions = 8usize;
        let max_hd = 3usize;
        // Generate random usizes
        let start_dimensions_seq = rng1.gen_iter::<usize>()
            // Select a random number of elements in the range [0,dimensions]
            .map(|i| sample(&mut rng2, 0..dimensions, i % dimensions))
            // Filter selections with less than the max tolerance
            .filter(|start_dimensions| start_dimensions.len() > max_hd);

        for start_dimensions in start_dimensions_seq.take(1000) {
            let mut p: SubstitutionDB<u64, u64> = Database::new(dimensions, max_hd);
            let a = 0b11111111u64;

            let mut b = a.clone();
            for start_dimension in start_dimensions.iter() {
                b = b & (0b10000000u64 >> *start_dimension);
            }

            let mut c: HashSet<u64> = HashSet::new();
            c.insert(a.clone());

            assert!(p.insert(a.clone()));

            let keys = p.get(&b);

            assert_eq!(None, keys);
        }
    }

    #[test]
    fn remove_inserted_key() {
        let mut p: SubstitutionDB<u64, u64> = Database::new(8, 2);
        let a = 0b00001111u64;

        p.insert(a.clone());

        assert!(p.remove(&a));

        let keys = p.get(&a);

        assert_eq!(None, keys);
    }

    #[test]
    fn remove_missing_key() {
        let mut p: SubstitutionDB<u64, u64> = Database::new(8, 2);
        let a = 0b00001111u64;

        assert!(!p.remove(&a));
    }

    /*
     * We want to simulate adding & removing a ton of keys and then verify the
     * state is consistent.  
    #[test]
    #[should_panic]
    fn stability_under_load() {
        // NOTE: we need a better way of coercing values - right now we only support
        // Vec<u8> - would be much better to implement a generic so we could set 
        // values directly.  IE, we need to convert u16 to [u8] here, and that's annoying
        let mut p: SubstitutionDB<InMemoryHashMapSet<u64, u64>> = Database::new(16, 4);

        let mut expected_present = [false; 65536];
        let mut expected_absent = [false; 65536];

        let mut rng = thread_rng();
        let seq = rng.gen_iter::<u16>();

        for i in seq.take(100000usize) {
            if expected_present[i] {
                p.remove(&(i as u64));
                expected_present[i] = false;
                expected_absent[i] = true;
            } else {
                p.insert(i as u64);
                expected_present[i] = true;
                expected_absent[i] = false;
            }

            if i % 1000 == 0 {
                //for i in 0..expected_present.len() {
                for i in 0..256 {
                    let mut found = false;
                    match p.get(&i) {
                        Some(set) => for key in set.iter() {
                            if *key == i as u64 {
                                found = true;
                            };
                        },
                        None => (),
                    }

                    assert!(found)
                }

                for i in 0usize..expected_absent.len() {
                    let mut found = false;
                    match p.get(&i) {
                        Some(set) => for key in set.iter() {
                            if *key == i as u64 {
                                found = true;
                            };
                        },
                        None => (),
                    }

                    assert!(!found)
                }
            }
        }
    }
     */

    #[test]
    fn idempotent_read() {
        fn prop(a: u64, b: u64, c: u64) -> quickcheck::TestResult {
            if a == c {
                // Removing C should also remove A, if they are the same
                return quickcheck::TestResult::discard()
            }

            let mut p: SubstitutionDB<u64, u64> = Database::new(64, 4);
            p.insert(a.clone());
            p.insert(b.clone());
            p.insert(c.clone());
            p.remove(&c);

            match p.get(&a) {
                Some(results) => quickcheck::TestResult::from_bool(results.contains(&a)),
                None => quickcheck::TestResult::failed(),
            }
        }
        quickcheck(prop as fn(u64, u64, u64) -> quickcheck::TestResult);
    }

    #[test]
    fn idempotent_delete() {
        fn prop(a: u64, b: u64, c: u64) -> quickcheck::TestResult {
            if a == c {
                // Removing C should also remove A, if they are the same
                return quickcheck::TestResult::discard()
            }

            let mut p: SubstitutionDB<u64, u64> = Database::new(64, 4);
            p.insert(a.clone());
            p.insert(b.clone());
            p.insert(c.clone());
            p.remove(&c);

            quickcheck::TestResult::from_bool(p.remove(&a))
        }
        quickcheck(prop as fn(u64, u64, u64) -> quickcheck::TestResult);
    }
}
