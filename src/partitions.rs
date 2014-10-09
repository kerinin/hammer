mod partitions {
    struct Partition {
        offsets: Tuple<int>,
        kv: KV,
    }

    impl Partition {
        fn get_key
    }

    struct Partitioning {
        max_hamming_distance: int,
        partitions: Tuple<Partition>,
    }

    impl Partitioning {
        fn get_keys<K, V>(&self, key: &K) -> Tuple<&V> {
            self.partitions.fold(Tuple<&V>, |set, x| set.union(x.get_keys(key)));
        }
    }
}
