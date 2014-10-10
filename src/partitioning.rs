use std::collections::{Map, MutableMap};
use std::hash::Hash;
use partition::{Partition}

trait ToBytes {
    fn to_bytes(&self) -> Vec(u8);
}

impl ToBytes for String {
    fn to_bytes(&self) -> Vec(u0) {
        return self.into_bytes();
    }
}
impl ToBytes for Vec(u8) {
    fn to_bytes(&self) -> Vec(u8) {
        return self;
    }
}
impl ToBytes for uint {
    fn to_bytes(&self) -> Vec(u8) {
        // Not sure if this is little or big -endian - def needs testing
        return range(0, self.BYTES).map(|&i| self.clone().Shl(8 * i).BitAnd(7) as u8);
    }
}

pub struct Partitioning<T> {
    max_hamming_distance: int,
    partitions: Vec<Partition<T>>,
}

impl<K: Hash + Eq, V, T: Map<K,V> + MutableMap<K,V>> Partitioning<K,V,T> {
    /*
     * Randomly partition the keyspace
     */
    fn new_rand(bits: uint, max_hamming_distance: uint) -> Partitioning<K,V,T> {
        let indices = range(0, bits).shuffle();
        step = bits / max_hamming_distance;

        let partitions = range(0, max_hamming_distance)
            .map(|&i| Partition<K,V,T> {offsets: indices[i, i + step], kv: T::new()} );

        return Partitioning<K,V,T> { max_hamming_distance: max_hamming_distance, partitions: partitions};
    }

    /*
     * Find all keys withing `hamming_distance` of `key`
     *
     */
    fn find_all(&self, key: &K) -> Set<&V> {
        let key_bytes = key.to_bytes();
        return self.partitions.flat_map(|&x| x.find(key_bytes)) as Set<V>;
    }

    /*
     * Insert `key` into indices
     */
    fn insert(&self, key: K) -> Vec<bool> {
        let key_bytes = key.to_bytes();
        return self.partitions.map(|&x| x.insert(key_bytes))
    }

    /*
     * Remove `key` from indices
     */
    fn remove(&self, key: &K) -> Vec<bool> {
        let key_bytes = key.to_bytes();
        return self.partitions.map(|&x| x.remove(key_bytes))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    //use super::{Indexable};

    #[test]
    fn the_truth() {
        assert!(true);
    }
}
