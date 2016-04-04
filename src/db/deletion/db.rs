use std::fmt;
use std::cmp::*;
use std::clone::*;
use std::collections::*;
use std::collections::hash_map::Entry::*;

use num::rational::Ratio;

use db::id_map;
use db::TypeMap;
use db::Database;
use db::result_accumulator::ResultAccumulator;
use db::map_set::{MapSet, InMemoryHash};
use db::window::{Window, Windowable};
use db::id_map::{ToID, IDMap};
use db::deletion::{Key, DeletionVariant, Dvec};

type TypeMapVecU8 = (Vec<u8>, id_map::HashMap<u64, Vec<u8>>, InMemoryHash<Key<Dvec>, u64>);

/// HmSearch Database using deletion variants
///
/// Pseudo-code Index(T):
/// 1. Build windows [W] from T
/// 2. Generate ID, store T -> ST[ID]
/// 3. (foreach W) generate variants [V]
/// 4. (foreach W, V) Add ID to ST[V]
///
/// Pseudo-code Query(Tq) -> [Tr]:
/// 1. Build windows [W] from Tq
/// 2. (foreach W) generate variants [V]
/// 3. (foreach W+V) Fetch ST[V] -> IDv
/// 4. Filter [IDv] -> [IDr]
/// 5. (foreach IDr) Fetch SV[IDr] -> Tr
///
pub struct DB<T = (Vec<u8>, id_map::HashMap<u64, Vec<u8>>, InMemoryHash<Key<Dvec>, u64>)> where T: TypeMap {
    pub dimensions: usize,
    pub tolerance: usize,
    pub partition_count: usize,
    pub partitions: Vec<Window>,

    value_store: <T as TypeMap>::ValueStore,
    variant_store: <T as TypeMap>::VariantStore,
}

impl<T: TypeMap> DB<T> where
<T as TypeMap>::ValueStore: Default,
<T as TypeMap>::VariantStore: Default,
<T as TypeMap>::Window: DeletionVariant<<T as TypeMap>::Variant>,
<T as TypeMap>::VariantStore: MapSet<Key<<T as TypeMap>::Variant>, <T as TypeMap>::Identifier>,
{

    /// Create a new DB with default backing store
    ///
    /// Partitions the keyspace as evenly as possible - all partitions
    /// will have either N or N-1 dimensions
    ///
    pub fn new(dimensions: usize, tolerance: usize) -> DB<T> {
        DB::with_stores(dimensions, tolerance, Default::default(), Default::default())
    }
}

impl<T: TypeMap> DB<T> where
<T as TypeMap>::Window: DeletionVariant<<T as TypeMap>::Variant>,
<T as TypeMap>::VariantStore: MapSet<Key<<T as TypeMap>::Variant>, <T as TypeMap>::Identifier>,
{
    /// Create a new DB with given backing store
    ///
    /// Partitions the keyspace as evenly as possible - all partitions
    /// will have either N or N-1 dimensions
    ///
    pub fn with_stores(dimensions: usize, tolerance: usize, value_store: <T as TypeMap>::ValueStore, variant_store: <T as TypeMap>::VariantStore) -> DB<T> {

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
        return DB {
            dimensions: dimensions,
            tolerance: tolerance,
            partition_count: partition_count,
            partitions: partitions,

            value_store: value_store,
            variant_store: variant_store,
        };
    }
}

impl<T: TypeMap> Database<<T as TypeMap>::Input> for  DB<T> where
<T as TypeMap>::Window: DeletionVariant<<T as TypeMap>::Variant>,
<T as TypeMap>::VariantStore: MapSet<Key<<T as TypeMap>::Variant>, <T as TypeMap>::Identifier>,
{
    /// Get all indexed values within `self.tolerance` hamming distance of `key`
    ///
    fn get(&self, key: &<T as TypeMap>::Input) -> Option<HashSet<<T as TypeMap>::Input>> {
        let mut results = ResultAccumulator::new(self.tolerance, key.clone());

        // Split across tasks?
        for window in self.partitions.iter() {
            let mut counts: HashMap<<T as TypeMap>::Identifier, usize> = HashMap::new();
            let transformed_key = key.window(window.start_dimension, window.dimensions);

            for variant in transformed_key.deletion_variants(window.dimensions) {
                match self.variant_store.get(&(window.clone(), variant)) {
                    Some(ids) => {
                        // Iterate through the values found in the deletion variant's set
                        for id in ids.iter() {
                            // Increment the key's count (this is sort of cumberson in Rust...)
                            match counts.entry(id.clone()) {
                                Occupied(mut entry) => { *entry.get_mut() += 1; },
                                Vacant(entry) => { entry.insert(1); },
                            }
                        }
                    },
                    None => (),
                }
            }

            for (id, count) in counts {
                if count > 2 {
                    results.insert_zero_variant(&self.value_store.get(id))
                } else {
                    results.insert_one_variant(&self.value_store.get(id))
                }
            }
        }

        results.found_values()
    }

    /// Insert `key` into indices
    ///
    /// Returns true if key was added to ANY index
    ///
    fn insert(&mut self, key: <T as TypeMap>::Input) -> bool {
        let id = key.clone().to_id();
        self.value_store.insert(id.clone(), key.clone());

        // Split across tasks?
        self.partitions.clone().into_iter().map(|window| {
            let transformed_key = key.window(window.start_dimension, window.dimensions);

            // NOTE: think about how to detect 'new' values
            transformed_key.deletion_variants(window.dimensions).map(|deletion_variant| {
                self.variant_store.insert((window.clone(), deletion_variant.clone()), id.clone())

            }).collect::<Vec<bool>>().iter().any(|i| *i)

            // Collecting first to force evaluation
        }).collect::<Vec<bool>>().iter().any(|i| *i)
    }

    /// Remove `key` from indices
    ///
    /// Returns true if key was removed from ANY index
    ///
    fn remove(&mut self, key: &<T as TypeMap>::Input) -> bool {
        let id = key.clone().to_id();
        self.value_store.remove(&id);

        // Split across tasks?
        self.partitions.clone().into_iter().map(|window| {
            let transformed_key = key.window(window.start_dimension, window.dimensions);

            transformed_key.deletion_variants(window.dimensions).map(|ref deletion_variant| {
                self.variant_store.remove(&(window.clone(), deletion_variant.clone()), &id)

            }).collect::<Vec<bool>>().iter().any(|i| *i)

            // Collecting first to force evaluation
        }).collect::<Vec<bool>>().iter().any(|i| *i)
    }
}

impl<T: TypeMap> fmt::Debug for DB<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}:{}:{})", self.dimensions, self.tolerance, self.partition_count)
    }
}

impl<T: TypeMap> PartialEq for DB<T> {
    fn eq(&self, other: &DB<T>) -> bool {
        return self.dimensions == other.dimensions &&
            self.tolerance == other.tolerance &&
            self.partition_count == other.partition_count;// &&
        //self.partitions.eq(&other.partitions);
    }

    fn ne(&self, other: &DB<T>) -> bool {
        return self.dimensions != other.dimensions ||
            self.tolerance != other.tolerance ||
            self.partition_count != other.partition_count; // ||
        //self.partitions.eq(&other.partitions);
    }
}

// Internal tests
#[test]
fn test_ddb_partition_evenly() {
    let a: DB<TypeMapVecU8> = DB::new(32, 5);

    assert_eq!(a.dimensions, 32);
    assert_eq!(a.tolerance, 5);
    assert_eq!(a.partition_count, 4);
    assert_eq!(a.partitions, vec![
               Window{start_dimension:0, dimensions: 8},
               Window{start_dimension:8, dimensions: 8},
               Window{start_dimension:16, dimensions: 8},
               Window{start_dimension:24, dimensions: 8}
    ]);
}


#[test]
fn test_ddb_partition_unevenly() {
    let a: DB<TypeMapVecU8> = DB::new(32, 7);

    assert_eq!(a.dimensions, 32);
    assert_eq!(a.tolerance, 7);
    assert_eq!(a.partition_count, 5);
    assert_eq!(a.partitions, vec![
               Window{start_dimension:0, dimensions: 7},
               Window{start_dimension:7, dimensions: 7},
               Window{start_dimension:14, dimensions: 6},
               Window{start_dimension:20, dimensions: 6},
               Window{start_dimension:26, dimensions: 6},
    ]);
}

#[test]
fn test_ddb_partition_too_many() {
    let a: DB<TypeMapVecU8> = DB::new(4, 8);

    assert_eq!(a.dimensions, 4);
    assert_eq!(a.tolerance, 8);
    assert_eq!(a.partition_count, 3);
    assert_eq!(a.partitions, vec![
               Window{start_dimension:0, dimensions: 2},
               Window{start_dimension:2, dimensions: 1},
               Window{start_dimension:3, dimensions: 1},
    ]);
}

#[test]
fn test_ddb_partition_zero() {
    let a: DB<TypeMapVecU8> = DB::new(32, 0);

    assert_eq!(a.dimensions, 32);
    assert_eq!(a.tolerance, 0);
    assert_eq!(a.partition_count, 1);
    assert_eq!(a.partitions, vec![
               Window{start_dimension:0, dimensions: 32},
    ]);
}

#[test]
fn test_ddb_partition_with_no_bytes() {
    let a: DB<TypeMapVecU8> = DB::new(0, 0);

    assert_eq!(a.dimensions, 0);
    assert_eq!(a.tolerance, 0);
    assert_eq!(a.partition_count, 1);
    assert_eq!(a.partitions, vec![
               Window{start_dimension:0, dimensions: 0},
    ]);
}

#[cfg(test)]
mod test {
    extern crate rand;
    extern crate quickcheck;
    extern crate bincode;
    extern crate rustc_serialize;

    use self::quickcheck::quickcheck;

    use std::collections::HashSet;
    use std::iter::repeat;
    use self::rand::{thread_rng, sample, Rng};
    use bincode::SizeLimit;
    use bincode::rustc_serialize::{encode};


    use db::*;
    use db::deletion::{DB};
    use db::deletion::db::{TypeMapVecU8};

    #[test]
    fn find_missing_key() {
        let p: DB<TypeMapVecU8> = DB::new(8, 2);
        let a = vec![0,0,0,0,0,0,0,0];
        let keys = p.get(&a);

        assert_eq!(None, keys);
    }

    #[test]
    fn insert_first_key() {
        let mut p: DB<TypeMapVecU8> = DB::new(8, 2);
        let a = vec![0,0,0,0,0,0,0,0];

        assert!(p.insert(a.clone()));
    }

    #[test]
    fn insert_second_key() {
        let mut p: DB<TypeMapVecU8> = DB::new(8, 2);
        let a = vec![0,0,0,0,0,0,0,0];

        p.insert(a.clone());

        assert!(!p.insert(a.clone()));
    }

    #[test]
    fn find_inserted_key() {
        let mut p: DB<TypeMapVecU8> = DB::new(8, 2);
        let a = vec![0,0,0,0,0,0,0,0];
        let mut b = HashSet::new();
        b.insert(a.clone());

        assert!(p.insert(a.clone()));

        let keys = p.get(&a);

        assert_eq!(Some(b), keys);
    }

    #[test]
    fn find_permutations_of_inserted_key() {
        let mut p: DB<TypeMapVecU8> = DB::new(8, 2);
        let a = vec![0,0,0,0,0,0,0,0];
        let b = vec![0,0,0,0,0,0,0,1];
        let mut c = HashSet::new();
        c.insert(a.clone());

        p.insert(a.clone());

        let keys = p.get(&b);

        assert_eq!(Some(c), keys);
    }

    #[test]
    fn find_permutations_of_multiple_similar_keys() {
        let mut p: DB<TypeMapVecU8> = DB::new(8, 4);
        let a = vec![0,0,0,0,0,0,0,0];
        let b = vec![1,0,0,0,0,0,0,0];
        let c = vec![1,0,0,0,0,0,0,1];
        let d = vec![1,1,0,0,0,0,0,1];
        let e = vec![1,1,0,0,0,0,1,1];
        let mut f = HashSet::new();
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
        let dimensions = 8;
        let max_hd = 3;
        let start_dimensions_seq = rng1.gen_iter::<usize>()
            .map(|i| sample(&mut rng2, 0..dimensions, i % max_hd));

        for start_dimensions in start_dimensions_seq.take(1000) {
            let mut p: DB<TypeMapVecU8> = DB::new(dimensions, max_hd);
            let a = repeat(1).take(dimensions).collect::<Vec<u8>>();

            let mut b = a.clone();
            for start_dimension in start_dimensions.iter() {
                b[*start_dimension] = 0;
            }

            let mut c = HashSet::new();
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
        let dimensions = 8;
        let max_hd = 3;
        // Generate random u64s
        let start_dimensions_seq = rng1.gen_iter::<usize>()
            // Select a random number of elements in the range [0,dimensions]
            .map(|i| sample(&mut rng2, 0..dimensions, i % dimensions))
            // Filter selections with less than the max tolerance
            .filter(|start_dimensions| start_dimensions.len() > max_hd);

        for start_dimensions in start_dimensions_seq.take(1000) {
            let mut p: DB<TypeMapVecU8> = DB::new(dimensions, max_hd);
            let a = repeat(1).take(dimensions).collect::<Vec<u8>>();

            let mut b = a.clone();
            for start_dimension in start_dimensions.iter() {
                b[*start_dimension] = 0;
            }

            let mut c = HashSet::new();
            c.insert(a.clone());

            assert!(p.insert(a.clone()));

            let keys = p.get(&b);

            assert_eq!(None, keys);
        }
    }

    #[test]
    fn remove_inserted_key() {
        let mut p: DB<TypeMapVecU8> = DB::new(8, 2);
        let a = vec![0,0,0,0,0,0,0,0];

        p.insert(a.clone());

        assert!(p.remove(&a));

        let keys = p.get(&a);

        assert_eq!(None, keys);
    }

    #[test]
    fn remove_missing_key() {
        let mut p: DB<TypeMapVecU8> = DB::new(8, 2);
        let a = vec![0,0,0,0,0,0,0,0];

        assert!(!p.remove(&a));
    }

    /*
     * We want to simulate adding & removing a ton of keys and then verify the
     * state is consistent.  
     */
    /*
    #[test]
    #[should_panic]
    fn stability_under_load() {
        // NOTE: we need a better way of coercing values - right now we only support
        // Vec<u8> - would be much better to implement a generic so we could set 
        // values directly.  IE, we need to convert u16 to [u8] here, and that's annoying
        let mut p: DB<TypeMapVecU8> = DB::new(16, 4);

        let mut expected_present = [false; 65536];
        let mut expected_absent = [false; 65536];

        let mut rng = thread_rng();
        let seq = rng.gen_iter::<u16>();

        for i in seq.take(100000) {
            if expected_present[i as usize] {
                p.remove(&(i as u64));
                expected_present[i as usize] = false;
                expected_absent[i as usize] = true;
            } else {
                p.insert(i as u64);
                expected_present[i as usize] = true;
                expected_absent[i as usize] = false;
            }

            if i % 1000 == 0 {
                //for i in 0..expected_present.len() {
                for i in 0u64..256u64 {
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

                for i in 0u64..expected_absent.len() as u64 {
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
                let avec: Vec<u8> = encode(&a, SizeLimit::Infinite).unwrap();
                let bvec: Vec<u8> = encode(&b, SizeLimit::Infinite).unwrap();
                let cvec: Vec<u8> = encode(&c, SizeLimit::Infinite).unwrap();

                let mut p: DB<TypeMapVecU8> = DB::new(8, 4);
                p.insert(avec.clone());
                p.insert(bvec.clone());
                p.insert(cvec.clone());
                p.remove(&cvec);

                match p.get(&avec) {
                    Some(results) => quickcheck::TestResult::from_bool(results.contains(&avec)),
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
                let avec: Vec<u8> = encode(&a, SizeLimit::Infinite).unwrap();
                let bvec: Vec<u8> = encode(&b, SizeLimit::Infinite).unwrap();
                let cvec: Vec<u8> = encode(&c, SizeLimit::Infinite).unwrap();

                let mut p: DB<TypeMapVecU8> = DB::new(8, 4);
                p.insert(avec.clone());
                p.insert(bvec.clone());
                p.insert(cvec.clone());
                p.remove(&cvec);

                quickcheck::TestResult::from_bool(p.remove(&avec))
            }
            quickcheck(prop as fn(u64, u64, u64) -> quickcheck::TestResult);
        }
    }
