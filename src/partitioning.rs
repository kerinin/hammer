extern crate num;

use std::collections::{HashMap,HashSet};
use std::cmp::{min,max};
use std::fmt;

use self::num::rational::Ratio;

use super::partition::{Partition};
use super::find_result::{FindResult, ZeroVariant, OneVariant};
use super::result_accumulator::ResultAccumulator;

struct Partitioning<T> {
    bits: uint,
    tolerance: uint,
    partition_count: uint,
    partitions: Vec<Partition<T>>,
}

impl<T> fmt::Show for Partitioning<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
        write!(f, "{}", self.partitions)
    }
}

impl<T: PartialEq> PartialEq for Partitioning<T> {
    fn eq(&self, other: &Partitioning<T>) -> bool {
        return self.partitions.eq(&other.partitions);
    }

    fn ne(&self, other: &Partitioning<T>) -> bool {
        return self.partitions.ne(&other.partitions);
    }
}

impl Partitioning<HashMap<Vec<u8>, Vec<u8>>> {
    /*
     * Partition the keyspace as evenly as possible
     */
    fn new(bits: uint, tolerance: uint) -> Partitioning<HashMap<Vec<u8>, Vec<u8>>> {

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

        let mut partitions: Vec<Partition<HashMap<Vec<u8>, Vec<u8>>>> = vec![];

        for i in range(0, head_count) {
            let shift = i * head_width;
            let mask = head_width;

            partitions.push(Partition::new(shift, mask));
        }

        for i in range(0, tail_count) {
            let shift = (head_count * head_width) + (i * tail_width);
            let mask = tail_width;

            partitions.push(Partition::new(shift, mask));
        }

        return Partitioning {
            bits: bits,
            tolerance: tolerance,
            partition_count: partition_count,
            partitions: partitions,
        };
    }

    fn find(&self, key: Vec<u8>) -> Option<HashSet<Vec<u8>>> {
        let mut results = ResultAccumulator::new(self.tolerance, key.clone());

        for partition in self.partitions.iter() {
            match partition.find(key.clone()) {
                Some(keys) => for k in keys.iter() {
                    results.merge(k);
                },
                None => (),
            }
        }

        return results.found_values()
    }

    /*
     * Insert `key` into indices
     */
    fn insert(&mut self, key: Vec<u8>) -> bool {
        return self.partitions.iter_mut()
            .any(|x| x.insert(key.clone()));
    }

    /*
     * Remove `key` from indices
     */
    fn remove(&mut self, key: Vec<u8>) -> bool {
        return self.partitions.iter_mut()
            .any(|x| x.remove(key.clone()));
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashSet,HashMap};
    use std::rand::{task_rng, sample, Rng};
    use std::iter::Repeat;
    use super::{Partitioning};
    use partition::{Partition};
    use permutable::{Permutable};

    #[test]
    fn partition_evenly() {
        let a: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(32, 5);
        let b = Partitioning {
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
        let a: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(32, 7);
        let b = Partitioning {
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
        let a: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(4, 8);
        let b = Partitioning {
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
        let a: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(32, 0);
        let b = Partitioning {
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
        let a: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(0, 0);
        let b = Partitioning {
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
        let p: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(8, 2);
        let a = vec![0b11111111u8];
        let keys = p.find(a);

        assert_eq!(None, keys);
    }

    #[test]
    fn insert_first_key() {
        let mut p: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(8, 2);
        let a = vec![0b11111111u8];

        assert!(p.insert(a.clone()));
    }

    #[test]
    fn insert_second_key() {
        let mut p: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(8, 2);
        let a = vec![0b11111111u8];

        assert!(!p.insert(a.clone()));
    }

    #[test]
    fn find_inserted_key() {
        let mut p: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(8, 2);
        let a = vec![0b11111111u8];
        let mut b: HashSet<Vec<u8>> = HashSet::new();
        b.insert(a.clone());

        assert!(p.insert(a.clone()));

        let keys = p.find(a.clone());

        assert_eq!(Some(b), keys);
    }

    #[test]
    fn find_permutations_of_inserted_key() {
        let mut p: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(8, 2);
        let a = vec![0b00001111u8];
        let b = vec![0b00000111u8];
        let mut c: HashSet<Vec<u8>> = HashSet::new();
        c.insert(a.clone());

        p.insert(a.clone());

        let keys = p.find(b.clone());

        assert_eq!(Some(c), keys);
    }

    #[test]
    fn find_permutations_of_multiple_similar_keys() {
        let mut p: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(8, 2);
        let a = vec![0b00000000u8];
        let b = vec![0b10000000u8];
        let c = vec![0b10000001u8];
        let d = vec![0b11000001u8];
        let e = vec![0b11000011u8];
        let mut f: HashSet<Vec<u8>> = HashSet::new();
        f.insert(b.clone());
        f.insert(c.clone());
        f.insert(d.clone());
        f.insert(e.clone());

        p.insert(b.clone());
        p.insert(c.clone());
        p.insert(d.clone());
        p.insert(e.clone());

        let keys = p.find(a.clone());

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
            let mut p: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(bits, max_hd);
            let a = vec![0b11111111u8];

            let mut b = a.clone();
            for shift in shifts.iter() {
                b = b.bitxor(&vec![0b10000000u8].shr(shift));
            }

            let mut c: HashSet<Vec<u8>> = HashSet::new();
            c.insert(a.clone());

            assert!(p.insert(a.clone()));

            let keys = p.find(b.clone());

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
            let mut p: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(bits, max_hd);
            let a = vec![0b11111111u8];

            let mut b = a.clone();
            for shift in shifts.iter() {
                b = b.bitxor(&vec![0b10000000u8].shr(shift));
            }

            let mut c: HashSet<Vec<u8>> = HashSet::new();
            c.insert(a.clone());

            assert!(p.insert(a.clone()));

            let keys = p.find(b.clone());

            assert_eq!(None, keys);
        }
    }

    #[test]
    fn remove_inserted_key() {
        let mut p: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(8, 2);
        let a = vec![0b00001111u8];

        p.insert(a.clone());

        assert!(p.remove(a.clone()));

        let keys = p.find(a.clone());

        assert_eq!(None, keys);
    }

    #[test]
    fn remove_missing_key() {
        let mut p: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(8, 2);
        let a = vec![0b00001111u8];

        assert!(!p.remove(a));
    }

    /*
     * We want to simulate adding & removing a ton of keys and then verify the
     * state is consistent.  
     */
    #[test]
    fn stability_under_load() {
        let mut p: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(16, 4);

        let mut expected_present: Vec<bool> = Vec::with_capacity(65536);
        let mut expected_absent: Vec<bool> = Vec::with_capacity(65536);

        let mut rng = task_rng();
        let seq = rng.gen_iter::<uint>();

        for i in seq.take(100000u) {
            if expected_present[i] {
                p.remove(vec![i as u8]);
                *expected_present.get_mut(i) = false;
                *expected_absent.get_mut(i) = true;
            } else {
                p.insert(vec![i as u8]);
                *expected_present.get_mut(i) = true;
                *expected_absent.get_mut(i) = false;
            }

            if i % 1000 == 0 {
                for i in range(0, expected_present.len()) {
                    let mut found = false;
                    let b = expected_present[i];
                    match p.find(vec![i as u8]) {
                        Some(set) => for key in set.iter() {
                            if *key == vec![i as u8] {
                                found = true;
                            };
                        },
                        None => (),
                    }

                    assert!(found)
                }

                for i in range(0, expected_absent.len()) {
                    let mut found = false;
                    let b = expected_present[i];
                    match p.find(vec![i as u8]) {
                        Some(set) => for key in set.iter() {
                            if *key == vec![i as u8] {
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
