pub mod substitution_db;
pub mod deletion_db;
pub mod value;
mod hash_map_set;
mod result_accumulator;

mod bench; // Uncomment to get benchmarks to run

use db::value::{Value, Window, SubstitutionVariant, DeletionVariant, Hamming};
use db::substitution_db::SubstitutionPartition;
use db::deletion_db::DeletionPartition;

pub struct SubstitutionDB<V> where V: Value + Window + SubstitutionVariant + Hamming {
    dimensions: usize,
    tolerance: usize,
    partition_count: usize,
    partitions: Vec<SubstitutionPartition<V>>,
}

pub struct DeletionDB<V> where V: Value + Window + DeletionVariant + Hamming {
    dimensions: usize,
    tolerance: usize,
    partition_count: usize,
    partitions: Vec<DeletionPartition<V>>,
}
