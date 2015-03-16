extern crate num;

use std::fmt;

use std::collections::HashSet;
use self::num::rational::Ratio;

use db::hash_map_set::HashMapSet;
use db::result_accumulator::ResultAccumulator;
use db::value::{Value, Window, SubstitutionVariant, Hamming};

pub struct SubstitutionPartition<V: Value> {
    pub start_dimension: usize,
    pub dimensions: usize,

    pub zero_kv: HashMapSet<V, V>,
    pub one_kv: HashMapSet<V, V>,
}

pub struct SubstitutionDatabase<V> where V: Value + Window + SubstitutionVariant + Hamming {
    dimensions: usize,
    tolerance: usize,
    partition_count: usize,
    partitions: Vec<SubstitutionPartition<V>>,
}

impl<V: Value> SubstitutionPartition<V> {
    pub fn new(start_dimension: usize, dimensions: usize) -> SubstitutionPartition<V> {
        let zero_kv: HashMapSet<V, V> = HashMapSet::new();
        let one_kv: HashMapSet<V, V> = HashMapSet::new();
        return SubstitutionPartition {start_dimension: start_dimension, dimensions: dimensions, zero_kv: zero_kv, one_kv: one_kv};
    }
}

impl<V: Value + Window + SubstitutionVariant + Hamming> SubstitutionDatabase<V> {
    /*
     * Partition the keyspace as evenly as possible
     */
    pub fn new(dimensions: usize, tolerance: usize) -> SubstitutionDatabase<V> {

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
        let mut partitions: Vec<SubstitutionPartition<V>> = Vec::with_capacity(head_count + tail_count);
        for i in 0..head_count {
            let start_dimension = i * head_width;
            let dimensions = head_width;
            let p: SubstitutionPartition<V> = SubstitutionPartition::new(start_dimension, dimensions);

            partitions.push(p);
        }
        for i in 0..tail_count {
            let start_dimension = (head_count * head_width) + (i * tail_width);
            let dimensions = tail_width;

            partitions.push(SubstitutionPartition::new(start_dimension, dimensions));
        }

        // Done!
        return SubstitutionDatabase {
            dimensions: dimensions,
            tolerance: tolerance,
            partition_count: partition_count,
            partitions: partitions,
        };
    }

    pub fn get(&self, key: &V) -> Option<HashSet<V>> {
        let mut results = ResultAccumulator::new(self.tolerance, key.clone());

        // Split across tasks?
        for partition in self.partitions.iter() {

            let transformed_key = &key.window(partition.start_dimension, partition.dimensions);
            match partition.zero_kv.get(transformed_key) {
                Some(keys) => {
                    for k in keys.iter() {
                        results.insert_zero_variant(k)
                    }
                },
                None => {},
            }

            match partition.one_kv.get(transformed_key) {
                Some(keys) => {
                    for k in keys.iter() {
                        results.insert_zero_variant(k)
                    }
                },
                None => {},
            }
        }

        results.found_values()
    }

    /*
     * Insert `key` into indices
     * Returns true if key was added to ANY index
     */
    pub fn insert(&mut self, key: V) -> bool {
        // Split across tasks?
        self.partitions.iter_mut().map(|ref mut partition| {
            let transformed_key = key.window(partition.start_dimension, partition.dimensions);

            if partition.zero_kv.insert(transformed_key.clone(), key.clone()) {
                for k in transformed_key.substitution_variants(partition.dimensions).iter() {
                    partition.one_kv.insert(k.clone(), key.clone());
                }
                true
            } else {
                false
            }

            // Collecting first to force evaluation
        }).collect::<Vec<bool>>().iter().any(|i| *i)
    }

    /*
     * Remove `key` from indices
     * Returns true if key was removed from ANY index
     */
    pub fn remove(&mut self, key: &V) -> bool {
        // Split across tasks?
        self.partitions.iter_mut().map(|ref mut partition| {
            let transformed_key = &key.window(partition.start_dimension, partition.dimensions);

            if partition.zero_kv.remove(transformed_key, key) {
                for k in transformed_key.substitution_variants(partition.dimensions).iter() {
                    partition.one_kv.remove(k, key);
                }
                true
            } else {
                false
            }

            // Collecting first to force evaluation
        }).collect::<Vec<bool>>().iter().any(|i| *i)
    }
}

impl<V: Value + Window + SubstitutionVariant + Hamming> fmt::Debug for SubstitutionDatabase<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}:{}:{})", self.dimensions, self.tolerance, self.partition_count)
    }
}

impl<V: Value + Window + SubstitutionVariant + Hamming> PartialEq for SubstitutionDatabase<V> {
    fn eq(&self, other: &SubstitutionDatabase<V>) -> bool {
        return self.dimensions == other.dimensions &&
            self.tolerance == other.tolerance &&
            self.partition_count == other.partition_count;// &&
            //self.partitions.eq(&other.partitions);
    }

    fn ne(&self, other: &SubstitutionDatabase<V>) -> bool {
        return self.dimensions != other.dimensions ||
            self.tolerance != other.tolerance ||
            self.partition_count != other.partition_count; // ||
            //self.partitions.eq(&other.partitions);
    }
}


#[cfg(test)]
mod test {
    extern crate rand;

    use std::collections::{HashSet};
    use self::rand::{thread_rng, sample, Rng};

    use db::substitution_database::{SubstitutionDatabase, SubstitutionPartition};

    #[test]
    fn partition_evenly() {
        let a: SubstitutionDatabase<usize> = SubstitutionDatabase::new(32, 5);
        let b = SubstitutionDatabase {
            dimensions: 32,
            tolerance: 5,
            partition_count: 4,
            partitions: vec![
                SubstitutionPartition::new(0, 8),
                SubstitutionPartition::new(8, 8),
                SubstitutionPartition::new(16, 8),
                SubstitutionPartition::new(24, 8)
                    ]};

        assert_eq!(a, b);
    }

    #[test]
    fn partition_unevenly() {
        let a: SubstitutionDatabase<usize> = SubstitutionDatabase::new(32, 7);
        let b = SubstitutionDatabase {
            dimensions: 32,
            tolerance: 7,
            partition_count: 5,
            partitions: vec![
                SubstitutionPartition::new(0, 7),
                SubstitutionPartition::new(7, 7),
                SubstitutionPartition::new(14, 6),
                SubstitutionPartition::new(20, 6),
                SubstitutionPartition::new(26, 6)
                    ]};

        assert_eq!(a, b);
    }

    #[test]
    fn partition_too_many() {
        let a: SubstitutionDatabase<usize> = SubstitutionDatabase::new(4, 8);
        let b = SubstitutionDatabase {
            dimensions: 4,
            tolerance: 8,
            partition_count: 3,
            partitions: vec![
                SubstitutionPartition::new(0, 2),
                SubstitutionPartition::new(2, 1),
                SubstitutionPartition::new(3, 1),
            ]
        };

        assert_eq!(a, b);
    }

    #[test]
    fn partition_zero() {
        let a: SubstitutionDatabase<usize> = SubstitutionDatabase::new(32, 0);
        let b = SubstitutionDatabase {
            dimensions: 32,
            tolerance: 0,
            partition_count: 1,
            partitions: vec![
                SubstitutionPartition::new(0, 32),
            ]
        };

        assert_eq!(a, b);
    }

    #[test]
    fn partition_with_no_bytes() {
        let a: SubstitutionDatabase<usize> = SubstitutionDatabase::new(0, 0);
        let b = SubstitutionDatabase {
            dimensions: 0,
            tolerance: 0,
            partition_count: 1,
            partitions: vec![
                SubstitutionPartition::new(0, 0),
            ]
        };

        assert_eq!(a, b);
    }

    #[test]
    fn find_missing_key() {
        let p: SubstitutionDatabase<usize> = SubstitutionDatabase::new(8, 2);
        let a = 0b11111111usize;
        let keys = p.get(&a);

        assert_eq!(None, keys);
    }

    #[test]
    fn insert_first_key() {
        let mut p: SubstitutionDatabase<usize> = SubstitutionDatabase::new(8, 2);
        let a = 0b11111111usize;

        assert!(p.insert(a.clone()));
    }

    #[test]
    fn insert_second_key() {
        let mut p: SubstitutionDatabase<usize> = SubstitutionDatabase::new(8, 2);
        let a = 0b11111111usize;

        p.insert(a.clone());

        assert!(!p.insert(a.clone()));
    }

    #[test]
    fn find_inserted_key() {
        let mut p: SubstitutionDatabase<usize> = SubstitutionDatabase::new(8, 2);
        let a = 0b11111111usize;
        let mut b: HashSet<usize> = HashSet::new();
        b.insert(a.clone());

        assert!(p.insert(a.clone()));

        let keys = p.get(&a);

        assert_eq!(Some(b), keys);
    }

    #[test]
    fn find_permutations_of_inserted_key() {
        let mut p: SubstitutionDatabase<usize> = SubstitutionDatabase::new(8, 2);
        let a = 0b00001111usize;
        let b = 0b00000111usize;
        let mut c = HashSet::new();
        c.insert(a.clone());

        p.insert(a.clone());

        let keys = p.get(&b);

        assert_eq!(Some(c), keys);
    }

    #[test]
    fn find_permutations_of_multiple_similar_keys() {
        let mut p: SubstitutionDatabase<usize> = SubstitutionDatabase::new(8, 4);
        let a = 0b00000000usize;
        let b = 0b10000000usize;
        let c = 0b10000001usize;
        let d = 0b11000001usize;
        let e = 0b11000011usize;
        let mut f: HashSet<usize> = HashSet::new();
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
            let mut p: SubstitutionDatabase<usize> = SubstitutionDatabase::new(dimensions, max_hd);
            let a = 0b11111111usize;

            let mut b = a.clone();
            for start_dimension in start_dimensions.iter() {
                b = b ^ (0b10000000usize >> *start_dimension);
            }

            let mut c: HashSet<usize> = HashSet::new();
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

        for start_dimensions in start_dimensions_seq.take(1000usize) {
            let mut p: SubstitutionDatabase<usize> = SubstitutionDatabase::new(dimensions, max_hd);
            let a = 0b11111111usize;

            let mut b = a.clone();
            for start_dimension in start_dimensions.iter() {
                b = b & (0b10000000usize >> *start_dimension);
            }

            let mut c: HashSet<usize> = HashSet::new();
            c.insert(a.clone());

            assert!(p.insert(a.clone()));

            let keys = p.get(&b);

            assert_eq!(None, keys);
        }
    }

    #[test]
    fn remove_inserted_key() {
        let mut p: SubstitutionDatabase<usize> = SubstitutionDatabase::new(8, 2);
        let a = 0b00001111usize;

        p.insert(a.clone());

        assert!(p.remove(&a));

        let keys = p.get(&a);

        assert_eq!(None, keys);
    }

    #[test]
    fn remove_missing_key() {
        let mut p: SubstitutionDatabase<usize> = SubstitutionDatabase::new(8, 2);
        let a = 0b00001111usize;

        assert!(!p.remove(&a));
    }

    /*
     * We want to simulate adding & removing a ton of keys and then verify the
     * state is consistent.  
     */
    #[test]
    #[should_panic]
    fn stability_under_load() {
        // NOTE: we need a better way of coercing values - right now we only support
        // Vec<u8> - would be much better to implement a generic so we could set 
        // values directly.  IE, we need to convert u16 to [u8] here, and that's annoying
        let mut p: SubstitutionDatabase<usize> = SubstitutionDatabase::new(16, 4);

        let mut expected_present = [false; 65536];
        let mut expected_absent = [false; 65536];

        let mut rng = thread_rng();
        let seq = rng.gen_iter::<u16>();

        for i in seq.take(100000usize) {
            if expected_present[i as usize] {
                p.remove(&(i as usize));
                expected_present[i as usize] = false;
                expected_absent[i as usize] = true;
            } else {
                p.insert(i as usize);
                expected_present[i as usize] = true;
                expected_absent[i as usize] = false;
            }

            if i % 1000 == 0 {
                //for i in 0..expected_present.len() {
                for i in 0usize..256usize {
                    let mut found = false;
                    match p.get(&i) {
                        Some(set) => for key in set.iter() {
                            if *key == i as usize {
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
                            if *key == i as usize {
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
}
