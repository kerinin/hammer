extern crate num;

use std::fmt;

use std::collections::HashSet;
use self::num::rational::Ratio;

use db::value::Value;
use db::partition::Partition;
//use db::result_accumulator::ResultAccumulator;
use db::find_result::FindResult;
use db::store::Store;

pub struct Database<S> {
    bits: uint,
    tolerance: uint,
    partition_count: uint,
    partitions: Vec<Partition<S>>,
}

impl<S> fmt::Show for Database<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}:{}:{})", self.bits, self.tolerance, self.partition_count)
    }
}

impl<V: PartialEq, S: Store<V, V>> PartialEq for Database<S> {
    fn eq(&self, other: &Database<S>) -> bool {
        return self.bits == other.bits &&
            self.tolerance == other.tolerance &&
            self.partition_count == other.partition_count;// &&
            //self.partitions.eq(&other.partitions);
    }

    fn ne(&self, other: &Database<S>) -> bool {
        return self.bits != other.bits ||
            self.tolerance != other.tolerance ||
            self.partition_count != other.partition_count; // ||
            //self.partitions.eq(&other.partitions);
    }
}

impl<V: Value, S: Store<V, V>> Database<S> {
    /*
     * Partition the keyspace as evenly as possible
     */
    pub fn new(bits: uint, tolerance: uint) -> Database<S> {

        let partition_count = if tolerance == 0 {
            1
        } else if tolerance > bits {
            (bits + 3) / 2
        } else {
            (tolerance + 3) / 2
        };

        let head_width = Ratio::new(bits, partition_count).ceil().to_integer() as uint;
        let tail_width = Ratio::new(bits, partition_count).floor().to_integer() as uint;

        let head_count = bits % partition_count;
        let tail_count = partition_count - head_count;

        let mut partitions: Vec<Partition<S>> = vec![];

        for i in range(0, head_count) {
            let shift = i * head_width;
            let mask = head_width;
            let p: Partition<S> = Partition::new(shift, mask);

            partitions.push(p);
        }

        for i in range(0, tail_count) {
            let shift = (head_count * head_width) + (i * tail_width);
            let mask = tail_width;

            partitions.push(Partition::new(shift, mask));
        }

        return Database {
            bits: bits,
            tolerance: tolerance,
            partition_count: partition_count,
            partitions: partitions,
        };
    }

    pub fn get(&mut self, key: V) -> Option<HashSet<V>> {
        /*
         * This is the method described in the HmSearch paper.  It's slower than
         * just checking the hamming distance, but I'm going to leave it commented
         * out here becase it may be necessary for vector-hamming distances (as
         * opposed to scalar-hamming distances).
         */
        //let mut results: ResultAccumulator<V> = ResultAccumulator::new(self.tolerance, key.clone());

        //for partition in self.partitions.iter() {
        //    let found = partition.find(key.clone());

        //    for k in found.iter() {
        //        results.merge(k);
        //    }
        //}

        //return results.found_values()

        let mut results: HashSet<V> = HashSet::new();

        for partition in self.partitions.iter_mut() {
            for result in partition.get(key.clone()).iter() {
                match *result {
                    FindResult::ZeroVariant(ref value) => {
                        if value.hamming(&key) <= self.tolerance { 
                            results.insert(value.clone());
                        };
                    },
                    FindResult::OneVariant(ref value) => {
                        if value.hamming(&key) <= self.tolerance {
                            results.insert(value.clone());
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

        for p in self.partitions.iter_mut() {
            inserted = p.insert(key.clone()) || inserted
        }

        inserted
    }

    /*
     * Remove `key` from indices
     * Returns true if key was removed from ANY index
     */
    pub fn remove(&mut self, key: V) -> bool {
        let mut removed = false;

        for p in self.partitions.iter_mut() {
            removed = p.remove(key.clone()) || removed
        }

        removed
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashSet};
    use std::rand::{task_rng, sample, Rng};

    use db::database::Database;
    use db::partition::Partition;
    use db::permutable::Permutable;
    use db::hash_map_set::HashMapSet;

    #[test]
    fn partition_evenly() {
        let a: Database<HashMapSet<uint, uint>> = Database::new(32, 5);
        let b = Database {
            bits: 32,
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
        let a: Database<HashMapSet<uint, uint>> = Database::new(32, 7);
        let b = Database {
            bits: 32,
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
        let a: Database<HashMapSet<uint, uint>> = Database::new(4, 8);
        let b = Database {
            bits: 4,
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
        let a: Database<HashMapSet<uint, uint>> = Database::new(32, 0);
        let b = Database {
            bits: 32,
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
        let a: Database<HashMapSet<uint, uint>> = Database::new(0, 0);
        let b = Database {
            bits: 0,
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
        let mut p: Database<HashMapSet<uint, uint>> = Database::new(8, 2);
        let a = 0b11111111u;
        let keys = p.get(a);

        assert_eq!(None, keys);
    }

    #[test]
    fn insert_first_key() {
        let mut p: Database<HashMapSet<uint, uint>> = Database::new(8, 2);
        let a = 0b11111111u;

        assert!(p.insert(a.clone()));
    }

    #[test]
    fn insert_second_key() {
        let mut p: Database<HashMapSet<uint, uint>> = Database::new(8, 2);
        let a = 0b11111111u;

        p.insert(a.clone());

        assert!(!p.insert(a.clone()));
    }

    #[test]
    fn find_inserted_key() {
        let mut p: Database<HashMapSet<uint, uint>> = Database::new(8, 2);
        let a = 0b11111111u;
        let mut b: HashSet<uint> = HashSet::new();
        b.insert(a.clone());

        assert!(p.insert(a.clone()));

        let keys = p.get(a.clone());

        assert_eq!(Some(b), keys);
    }

    #[test]
    fn find_permutations_of_inserted_key() {
        let mut p: Database<HashMapSet<uint, uint>> = Database::new(8, 2);
        let a = 0b00001111u;
        let b = 0b00000111u;
        let mut c = HashSet::new();
        c.insert(a.clone());

        p.insert(a.clone());

        let keys = p.get(b.clone());

        assert_eq!(Some(c), keys);
    }

    #[test]
    fn find_permutations_of_multiple_similar_keys() {
        let mut p: Database<HashMapSet<uint, uint>> = Database::new(8, 4);
        let a = 0b00000000u;
        let b = 0b10000000u;
        let c = 0b10000001u;
        let d = 0b11000001u;
        let e = 0b11000011u;
        let mut f: HashSet<uint> = HashSet::new();
        f.insert(b.clone());
        f.insert(c.clone());
        f.insert(d.clone());
        f.insert(e.clone());

        p.insert(b.clone());
        p.insert(c.clone());
        p.insert(d.clone());
        p.insert(e.clone());

        let keys = p.get(a.clone());

        assert_eq!(Some(f), keys);
    }

    #[test]
    fn find_permutation_of_inserted_key() {
        let mut rng1 = task_rng();
        let mut rng2 = task_rng();
        let bits = 8u;
        let max_hd = 3u;
        let shifts_seq = rng1.gen_iter::<uint>()
            .map(|i| sample(&mut rng2, range(0, bits), i % max_hd));

        for shifts in shifts_seq.take(1000u) {
            let mut p: Database<HashMapSet<uint, uint>> = Database::new(bits, max_hd);
            let a = 0b11111111u;

            let mut b = a.clone();
            for shift in shifts.iter() {
                b = b.p_bitxor(&0b10000000u.p_shr(shift));
            }

            let mut c: HashSet<uint> = HashSet::new();
            c.insert(a.clone());

            assert!(p.insert(a.clone()));

            let keys = p.get(b.clone());

            assert_eq!(Some(c), keys);
        }
    }

    #[test]
    fn dont_find_permutation_of_inserted_key() {
        let mut rng1 = task_rng();
        let mut rng2 = task_rng();
        let bits = 8u;
        let max_hd = 3u;
        // Generate random uints
        let shifts_seq = rng1.gen_iter::<uint>()
            // Select a random number of elements in the range [0,bits]
            .map(|i| sample(&mut rng2, range(0, bits), i % bits))
            // Filter selections with less than the max tolerance
            .filter(|shifts| shifts.len() > max_hd);

        for shifts in shifts_seq.take(1000u) {
            let mut p: Database<HashMapSet<uint, uint>> = Database::new(bits, max_hd);
            let a = 0b11111111u;

            let mut b = a.clone();
            for shift in shifts.iter() {
                b = b.p_bitxor(&0b10000000u.p_shr(shift));
            }

            let mut c: HashSet<uint> = HashSet::new();
            c.insert(a.clone());

            assert!(p.insert(a.clone()));

            let keys = p.get(b.clone());

            assert_eq!(None, keys);
        }
    }

    #[test]
    fn remove_inserted_key() {
        let mut p: Database<HashMapSet<uint, uint>> = Database::new(8, 2);
        let a = 0b00001111u;

        p.insert(a.clone());

        assert!(p.remove(a.clone()));

        let keys = p.get(a.clone());

        assert_eq!(None, keys);
    }

    #[test]
    fn remove_missing_key() {
        let mut p: Database<HashMapSet<uint, uint>> = Database::new(8, 2);
        let a = 0b00001111u;

        assert!(!p.remove(a));
    }

    /*
     * We want to simulate adding & removing a ton of keys and then verify the
     * state is consistent.  
     */
    #[test]
    #[should_fail]
    fn stability_under_load() {
        // NOTE: we need a better way of coercing values - right now we only support
        // Vec<u8> - would be much better to implement a generic so we could set 
        // values directly.  IE, we need to convert u16 to [u8] here, and that's annoying
        let mut p: Database<HashMapSet<uint, uint>> = Database::new(16, 4);

        let mut expected_present = [false, ..65536];
        let mut expected_absent = [false, ..65536];

        let mut rng = task_rng();
        let seq = rng.gen_iter::<u16>();

        for i in seq.take(100000u) {
            if expected_present[i as uint] {
                p.remove(i as uint);
                expected_present[i as uint] = false;
                expected_absent[i as uint] = true;
            } else {
                p.insert(i as uint);
                expected_present[i as uint] = true;
                expected_absent[i as uint] = false;
            }

            if i % 1000 == 0 {
                //for i in range(0, expected_present.len()) {
                for i in range(0u, 256u) {
                    let mut found = false;
                    match p.get(i as uint) {
                        Some(set) => for key in set.iter() {
                            if *key == i as uint {
                                found = true;
                            };
                        },
                        None => (),
                    }

                    assert!(found)
                }

                for i in range(0u, expected_absent.len()) {
                    let mut found = false;
                    match p.get(i as uint) {
                        Some(set) => for key in set.iter() {
                            if *key == i as uint {
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
