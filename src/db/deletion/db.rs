use std::fmt;
use std::cmp::*;
use std::clone::*;
use std::hash::*;
use std::collections::*;
use std::collections::hash_map::Entry::*;
use std::marker::PhantomData;

use num::rational::Ratio;

use db::Database;
use db::result_accumulator::ResultAccumulator;
use db::map_set::{MapSet, InMemoryHash};
use db::hamming::Hamming;
use db::window::{Window, Windowable};
use db::id_map::{ToID, IDMap, Echo};
use db::deletion::{Key, DeletionVariant, Du64};

/// HmSearch Database using deletion variants
///
/// T: The data type being indexed - Database::Value
/// W: The type of windows over T - T: Window<W>.  Window types must be large
///   enough to store dimensions/tolerance  dimensions of T (ideally not larger)
/// V: The type of variants computed over windows - W: DeletionVariant<V>.
///   Generally should be (T, u8) unless you're working with large dimensions.
/// ID: Value identifier - balances memory use with collision probability given
///   the cardinality of the data being indexed
/// ST: The value sture - maps ID -> T
/// SV: The variant store - maps V -> ID
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
pub struct DB<T, W, V, ID = T, ST = Echo<T>, SV = InMemoryHash<Key<V>, T>> {
    value: PhantomData<T>,
    window: PhantomData<W>,
    variant: PhantomData<V>,
    id: PhantomData<ID>,

    pub dimensions: usize,
    pub tolerance: usize,
    pub partition_count: usize,
    pub partitions: Vec<Window>,

    value_store: ST,
    variant_store: SV,
}

impl<T, W, V> DB<T, W, V, T, Echo<T>, InMemoryHash<Key<V>, T>> where
T: Sync + Send + Clone + Eq + Hash + Hamming + Windowable<W>,
W: DeletionVariant<V>,
V: Sync + Send + Clone + Eq + Hash,
{

    /// Create a new DB with default backing store
    ///
    /// Partitions the keyspace as evenly as possible - all partitions
    /// will have either N or N-1 dimensions
    ///
    pub fn new(dimensions: usize, tolerance: usize) -> DB<T, W, V, T, Echo<T>, InMemoryHash<Key<V>, T>> {
        DB::with_stores(dimensions, tolerance, Echo::new(), InMemoryHash::new())
    }
}

impl<T, W, V, ID, ST, SV> DB<T, W, V, ID, ST, SV> where
T: Clone + Eq + Hash + Hamming + Windowable<W> + ToID<ID>,
W: DeletionVariant<V>,
V: Clone + Eq + Hash,
ID: Clone + Eq + Hash,
ST: IDMap<ID, T>,
SV: MapSet<Key<V>, ID>, 
{
    /// Create a new DB with given backing store
    ///
    /// Partitions the keyspace as evenly as possible - all partitions
    /// will have either N or N-1 dimensions
    ///
    pub fn with_stores(dimensions: usize, tolerance: usize, value_store: ST, variant_store: SV) -> DB<T, W, V, ID, ST, SV> {

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
            value: PhantomData,
            window: PhantomData,
            variant: PhantomData,
            id: PhantomData,

            dimensions: dimensions,
            tolerance: tolerance,
            partition_count: partition_count,
            partitions: partitions,

            value_store: value_store,
            variant_store: variant_store,
        };
    }
}

impl<T, W, V, ID, ST, SV> Database<T> for  DB<T, W, V, ID, ST, SV> where
T: Sync + Send + Clone + Eq + Hash + Hamming + Windowable<W> + ToID<ID>,
W: Sync + Send + DeletionVariant<V>,
V: Sync + Send + Clone + Eq + Hash,
ID: Sync + Send + Clone + Eq + Hash,
ST: IDMap<ID, T>,
SV: MapSet<Key<V>, ID>, 
{
    /// Get all indexed values within `self.tolerance` hamming distance of `key`
    ///
    fn get(&self, key: &T) -> Option<HashSet<T>> {
        let mut results = ResultAccumulator::new(self.tolerance, key.clone());

        // Split across tasks?
        for window in self.partitions.iter() {
            let mut counts: HashMap<ID, usize> = HashMap::new();
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
    fn insert(&mut self, key: T) -> bool {
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
    fn remove(&mut self, key: &T) -> bool {
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

impl<T, W, V, ID, ST, SV> fmt::Debug for DB<T, W, V, ID, ST, SV> where
T: Sync + Send + Clone + Eq + Hash + Hamming + Windowable<W> + ToID<ID>,
W: Sync + Send + DeletionVariant<V>,
V: Sync + Send + Clone + Eq + Hash,
ID: Sync + Send + Clone + Eq + Hash,
ST: IDMap<ID, T>,
SV: MapSet<Key<V>, ID>, 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}:{}:{})", self.dimensions, self.tolerance, self.partition_count)
    }
}

impl<T, W, V, ID, ST, SV> PartialEq for DB<T, W, V, ID, ST, SV> where
T: Clone + Eq + Hash,
V: Clone + Eq + Hash,
{
    fn eq(&self, other: &DB<T, W, V, ID, ST, SV>) -> bool {
        return self.dimensions == other.dimensions &&
            self.tolerance == other.tolerance &&
            self.partition_count == other.partition_count;// &&
        //self.partitions.eq(&other.partitions);
    }

    fn ne(&self, other: &DB<T, W, V, ID, ST, SV>) -> bool {
        return self.dimensions != other.dimensions ||
            self.tolerance != other.tolerance ||
            self.partition_count != other.partition_count; // ||
        //self.partitions.eq(&other.partitions);
    }
}

// Internal tests
#[test]
fn test_ddb_partition_evenly() {
    let a: DB<u64, u64, Du64> = DB::new(32, 5);

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
    let a: DB<u64, u64, Du64> = DB::new(32, 7);

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
    let a: DB<u64, u64, Du64> = DB::new(4, 8);

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
    let a: DB<u64, u64, Du64> = DB::new(32, 0);

    assert_eq!(a.dimensions, 32);
    assert_eq!(a.tolerance, 0);
    assert_eq!(a.partition_count, 1);
    assert_eq!(a.partitions, vec![
               Window{start_dimension:0, dimensions: 32},
    ]);
}

#[test]
fn test_ddb_partition_with_no_bytes() {
    let a: DB<u64, u64, Du64> = DB::new(0, 0);

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

    use self::quickcheck::quickcheck;

    use std::collections::HashSet;
    use self::rand::{thread_rng, sample, Rng};

    use db::*;
    use db::deletion::{DB};
    use db::deletion::{Du64};

    #[test]
    fn find_missing_key() {
        let p: DB<u64, u64, Du64> = DB::new(8, 2);
        let a = 0b11111111u64;
        let keys = p.get(&a);

        assert_eq!(None, keys);
    }

    #[test]
    fn insert_first_key() {
        let mut p: DB<u64, u64, Du64> = DB::new(8, 2);
        let a = 0b11111111u64;

        assert!(p.insert(a.clone()));
    }

    #[test]
    fn insert_second_key() {
        let mut p: DB<u64, u64, Du64> = DB::new(8, 2);
        let a = 0b11111111u64;

        p.insert(a.clone());

        assert!(!p.insert(a.clone()));
    }

    #[test]
    fn find_inserted_key() {
        let mut p: DB<u64, u64, Du64> = DB::new(8, 2);
        let a = 0b11111111u64;
        let mut b = HashSet::new();
        b.insert(a.clone());

        assert!(p.insert(a.clone()));

        let keys = p.get(&a);

        assert_eq!(Some(b), keys);
    }

    #[test]
    fn find_permutations_of_inserted_key() {
        let mut p: DB<u64, u64, Du64> = DB::new(8, 2);
        let a = 0b00001111u64;
        let b = 0b00000111u64;
        let mut c = HashSet::new();
        c.insert(a.clone());

        p.insert(a.clone());

        let keys = p.get(&b);

        assert_eq!(Some(c), keys);
    }

    #[test]
    fn find_permutations_of_multiple_similar_keys() {
        let mut p: DB<u64, u64, Du64> = DB::new(8, 4);
        let a = 0b00000000u64;
        let b = 0b10000000u64;
        let c = 0b10000001u64;
        let d = 0b11000001u64;
        let e = 0b11000011u64;
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
            let mut p: DB<u64, u64, Du64> = DB::new(dimensions, max_hd);
            let a = 0b11111111u64;

            let mut b = a.clone();
            for start_dimension in start_dimensions.iter() {
                b = b ^ (0b10000000u64 >> *start_dimension);
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
            let mut p: DB<u64, u64, Du64> = DB::new(dimensions, max_hd);
            let a = 0b11111111u64;

            let mut b = a.clone();
            for start_dimension in start_dimensions.iter() {
                b = b & (0b10000000u64 >> *start_dimension);
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
        let mut p: DB<u64, u64, Du64> = DB::new(8, 2);
        let a = 0b00001111u64;

        p.insert(a.clone());

        assert!(p.remove(&a));

        let keys = p.get(&a);

        assert_eq!(None, keys);
    }

    #[test]
    fn remove_missing_key() {
        let mut p: DB<u64, u64, Du64> = DB::new(8, 2);
        let a = 0b00001111u64;

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
        let mut p: DB<u64, u64, Du64> = DB::new(16, 4);

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

        #[test]
        fn idempotent_read() {
            fn prop(a: u64, b: u64, c: u64) -> quickcheck::TestResult {
                if a == c {
                    // Removing C should also remove A, if they are the same
                    return quickcheck::TestResult::discard()
                }

                let mut p: DB<u64, u64, Du64> = DB::new(64, 4);
                p.insert(a.clone());
                p.insert(b.clone());
                p.insert(c.clone());
                p.remove(&c);

                match p.get(&a) {
                    Some(results) => quickcheck::TestResult::from_bool(results.contains(&a)),
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

                let mut p: DB<u64, u64, Du64> = DB::new(64, 4);
                p.insert(a.clone());
                p.insert(b.clone());
                p.insert(c.clone());
                p.remove(&c);

                quickcheck::TestResult::from_bool(p.remove(&a))
            }
            quickcheck(prop as fn(u64, u64, u64) -> quickcheck::TestResult);
        }
    }
