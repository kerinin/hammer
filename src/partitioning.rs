extern crate num;

use std::collections::{HashMap};
use std::cmp::{min,max};
use std::fmt;

use self::num::rational::Ratio;

use super::partition::{Partition};

struct Partitioning<T> {
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
    fn new(bits: uint, max_hamming_distance: uint) -> Partitioning<HashMap<Vec<u8>, Vec<u8>>> {

        let partition_count = max(1, min(bits, max_hamming_distance + 1));

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

        return Partitioning {partitions: partitions};
    }

    /*
     * Find all keys withing `hamming_distance` of `key`
     *
    fn find_all(&self, key: &K) -> Set<&V> {
        let key_bytes = key.to_bytes();
        return self.partitions.flat_map(|&x| x.find(key_bytes)) as Set<V>;
    }
     */

    /*
     * Insert `key` into indices
    fn insert(&self, key: K) -> Vec<bool> {
        let key_bytes = key.to_bytes();
        return self.partitions.map(|&x| x.insert(key_bytes))
    }
     */

    /*
     * Remove `key` from indices
    fn remove(&self, key: &K) -> Vec<bool> {
        let key_bytes = key.to_bytes();
        return self.partitions.map(|&x| x.remove(key_bytes))
    }
     */
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use super::{Partitioning};
    use partition::{Partition};

    #[test]
    fn partition_evenly() {
        let a: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(32, 3);
        let b = Partitioning {partitions: vec![
            Partition::new(0, 8),
            Partition::new(8, 8),
            Partition::new(16, 8),
            Partition::new(24, 8)
            ]};

        assert_eq!(a, b);
    }

    #[test]
    fn partition_unevenly() {
        let a: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(32, 4);
        let b = Partitioning {partitions: vec![
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
        let b = Partitioning {partitions: vec![
            Partition::new(0, 1),
            Partition::new(1, 1),
            Partition::new(2, 1),
            Partition::new(3, 1),
            ]};

        assert_eq!(a, b);
    }

    #[test]
    fn partition_zero() {
        let a: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(32, 0);
        let b = Partitioning {partitions: vec![
            Partition::new(0, 32),
            ]};

        assert_eq!(a, b);
    }

    #[test]
    fn partition_with_no_bytes() {
        let a: Partitioning<HashMap<Vec<u8>, Vec<u8>>> = Partitioning::new(0, 0);
        let b = Partitioning {partitions: vec![
            Partition::new(0, 0),
            ]};

        assert_eq!(a, b);
    }
}
