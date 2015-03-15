extern crate num;

use std::fmt;

use std::collections::HashSet;
use self::num::rational::Ratio;

use db::partition::Partition;
//use db::result_accumulator::ResultAccumulator;
use db::find_result::FindResult;
use db::value::Value;

pub struct Database<V: Value> {
    dimensions: usize,
    tolerance: usize,
    partition_count: usize,
    partitions: Vec<Partition<V>>,
}


impl<V: Value> Database<V> {
    /*
     * Partition the keyspace as evenly as possible
     */
    pub fn new(dimensions: usize, tolerance: usize) -> Database<V> {

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
        let mut partitions: Vec<Partition<V>> = Vec::with_capacity(head_count + tail_count);
        for i in 0..head_count {
            let start_dimension = i * head_width;
            let dimensions = head_width;
            let p: Partition<V> = Partition::new(start_dimension, dimensions);

            partitions.push(p);
        }
        for i in 0..tail_count {
            let start_dimension = (head_count * head_width) + (i * tail_width);
            let dimensions = tail_width;

            partitions.push(Partition::new(start_dimension, dimensions));
        }

        // Done!
        return Database {
            dimensions: dimensions,
            tolerance: tolerance,
            partition_count: partition_count,
            partitions: partitions,
        };
    }

    pub fn get(&self, key: &V) -> Option<HashSet<V>> {
        /*
         * This is the method described in the HmSearch paper.  It's slower than
         * just checking the hamming distance, but I'm going to leave it commented
         * out here becase it may be necessary for vector-hamming distances (as
         * opposed to scalar-hamming distances).
         */
        //let mut results: ResultAccumulator = ResultAccumulator::new(self.tolerance, key.clone());

        //for partition in self.partitions.iter() {
        //    let found = partition.find(key.clone());

        //    for k in found.iter() {
        //        results.merge(k);
        //    }
        //}

        //return results.found_values()

        let mut results: HashSet<V> = HashSet::new();

        // Split across tasks?
        for partition in self.partitions.iter() {
            for result in partition.get(key).into_iter() {
                match result {
                    FindResult::ZeroVariant(value) => {
                        if value.hamming(key) <= self.tolerance { 
                            results.insert(value);
                        };
                    },
                    FindResult::OneVariant(value) => {
                        if value.hamming(key) <= self.tolerance {
                            results.insert(value);
                        };
                    }
                }
            }
        }

        if results.is_empty() {
            None
        } else {
            Some(results)
        }
    }

    /*
     * Insert `key` into indices
     * Returns true if key was added to ANY index
     */
    pub fn insert(&mut self, key: V) -> bool {
        let mut inserted = false;

        // Split across tasks?
        for p in self.partitions.iter_mut() {
            inserted = p.insert(key.clone()) || inserted
        }

        inserted
    }

    /*
     * Remove `key` from indices
     * Returns true if key was removed from ANY index
     */
    pub fn remove(&mut self, key: &V) -> bool {
        let mut removed = false;

        // Split across tasks?
        for p in self.partitions.iter_mut() {
            removed = p.remove(key) || removed
        }

        removed
    }
}

#[cfg(test)]
mod test {
    extern crate rand;

    use std::collections::{HashSet};
    use self::rand::{thread_rng, sample, Rng};

    use db::database::Database;
    use db::partition::Partition;

    #[test]
    fn partition_evenly() {
        let a: Database<usize> = Database::new(32, 5);
        let b = Database {
            dimensions: 32,
            tolerance: 5,
            partition_count: 4,
            partitions: vec![
                Partition::new(0, 8),
                Partition::new(8, 8),
                Partition::new(16, 8),
                Partition::new(24, 8)
                    ]};

        assert_eq!(a, b);
    }

    #[test]
    fn partition_unevenly() {
        let a: Database<usize> = Database::new(32, 7);
        let b = Database {
            dimensions: 32,
            tolerance: 7,
            partition_count: 5,
            partitions: vec![
                Partition::new(0, 7),
                Partition::new(7, 7),
                Partition::new(14, 6),
                Partition::new(20, 6),
                Partition::new(26, 6)
                    ]};

        assert_eq!(a, b);
    }

    #[test]
    fn partition_too_many() {
        let a: Database<usize> = Database::new(4, 8);
        let b = Database {
            dimensions: 4,
            tolerance: 8,
            partition_count: 3,
            partitions: vec![
                Partition::new(0, 2),
                Partition::new(2, 1),
                Partition::new(3, 1),
            ]
        };

        assert_eq!(a, b);
    }

    #[test]
    fn partition_zero() {
        let a: Database<usize> = Database::new(32, 0);
        let b = Database {
            dimensions: 32,
            tolerance: 0,
            partition_count: 1,
            partitions: vec![
                Partition::new(0, 32),
            ]
        };

        assert_eq!(a, b);
    }

    #[test]
    fn partition_with_no_bytes() {
        let a: Database<usize> = Database::new(0, 0);
        let b = Database {
            dimensions: 0,
            tolerance: 0,
            partition_count: 1,
            partitions: vec![
                Partition::new(0, 0),
            ]
        };

        assert_eq!(a, b);
    }

    #[test]
    fn find_missing_key() {
        let p: Database<usize> = Database::new(8, 2);
        let a = 0b11111111usize;
        let keys = p.get(&a);

        assert_eq!(None, keys);
    }

    #[test]
    fn insert_first_key() {
        let mut p: Database<usize> = Database::new(8, 2);
        let a = 0b11111111usize;

        assert!(p.insert(a.clone()));
    }

    #[test]
    fn insert_second_key() {
        let mut p: Database<usize> = Database::new(8, 2);
        let a = 0b11111111usize;

        p.insert(a.clone());

        assert!(!p.insert(a.clone()));
    }

    #[test]
    fn find_inserted_key() {
        let mut p: Database<usize> = Database::new(8, 2);
        let a = 0b11111111usize;
        let mut b: HashSet<usize> = HashSet::new();
        b.insert(a.clone());

        assert!(p.insert(a.clone()));

        let keys = p.get(&a);

        assert_eq!(Some(b), keys);
    }

    #[test]
    fn find_permutations_of_inserted_key() {
        let mut p: Database<usize> = Database::new(8, 2);
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
        let mut p: Database<usize> = Database::new(8, 4);
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
            let mut p: Database<usize> = Database::new(dimensions, max_hd);
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
            let mut p: Database<usize> = Database::new(dimensions, max_hd);
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
        let mut p: Database<usize> = Database::new(8, 2);
        let a = 0b00001111usize;

        p.insert(a.clone());

        assert!(p.remove(&a));

        let keys = p.get(&a);

        assert_eq!(None, keys);
    }

    #[test]
    fn remove_missing_key() {
        let mut p: Database<usize> = Database::new(8, 2);
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
        let mut p: Database<usize> = Database::new(16, 4);

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

impl<V: Value> fmt::Debug for Database<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}:{}:{})", self.dimensions, self.tolerance, self.partition_count)
    }
}

impl<V: Value> PartialEq for Database<V> {
    fn eq(&self, other: &Database<V>) -> bool {
        return self.dimensions == other.dimensions &&
            self.tolerance == other.tolerance &&
            self.partition_count == other.partition_count;// &&
            //self.partitions.eq(&other.partitions);
    }

    fn ne(&self, other: &Database<V>) -> bool {
        return self.dimensions != other.dimensions ||
            self.tolerance != other.tolerance ||
            self.partition_count != other.partition_count; // ||
            //self.partitions.eq(&other.partitions);
    }
}

