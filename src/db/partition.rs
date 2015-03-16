use std::fmt;

use db::hash_map_set::HashMapSet;
use db::value::Value;

pub struct Partition<V: Value> {
    pub start_dimension: usize,
    pub dimensions: usize,

    pub zero_kv: HashMapSet<V, V>,
    pub one_kv: HashMapSet<V, V>,
}

impl<V: Value> fmt::Debug for Partition<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "<Partition s{} m{}>", self.start_dimension, self.dimensions)
    }
}

impl<V: Value> PartialEq for Partition<V> {
    fn eq(&self, other: &Partition<V>) -> bool {
        return self.start_dimension.eq(&other.start_dimension) &&
            self.dimensions.eq(&other.dimensions); // &&
            //self.zero_kv.eq(&other.zero_kv) &&
            //self.one_kv.eq(&other.one_kv);
    }

    fn ne(&self, other: &Partition<V>) -> bool {
        return self.start_dimension.ne(&other.start_dimension) ||
            self.dimensions.ne(&other.dimensions); // ||
            //self.zero_kv.ne(&other.zero_kv) ||
            //self.one_kv.ne(&other.one_kv);
    }
}

impl<V: Value> Partition<V> {
    pub fn new(start_dimension: usize, dimensions: usize) -> Partition<V> {
        let zero_kv: HashMapSet<V, V> = HashMapSet::new();
        let one_kv: HashMapSet<V, V> = HashMapSet::new();
        return Partition {start_dimension: start_dimension, dimensions: dimensions, zero_kv: zero_kv, one_kv: one_kv};
    }
}
