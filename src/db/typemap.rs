//! TypeMap implementations for common types

use std::path::PathBuf;
use num::rational::Ratio;

use db::id_map;
use db::map_set;
use db::deletion;
use db::substitution;
use db::{TypeMap, StorageBackend, Factory, Database};

macro_rules! deletion_inmemory {
    ($t:ident, $elem:ty) => {
        pub type $t = ($elem, id_map::HashMap<u64, $elem>, map_set::InMemoryHash<deletion::Key<deletion::Dvec>, u64>);
        impl TypeMap for $t {
            type Input = $elem;
            type Window = $elem;
            type Variant = deletion::Dvec;
            type Identifier = u64;
            type ValueStore = id_map::HashMap<u64, $elem>;
            type VariantStore = map_set::InMemoryHash<deletion::Key<deletion::Dvec>, u64>;
        }
    }
}

macro_rules! deletion_temp_rocksdb {
    ($t:ident, $elem:ty) => {
        pub type $t = ($elem, id_map::TempRocksDB<u64, $elem>, map_set::TempRocksDB<deletion::Key<deletion::Dvec>, u64>);
        impl TypeMap for $t {
            type Input = $elem;
            type Window = $elem;
            type Variant = deletion::Dvec;
            type Identifier = u64;
            type ValueStore = id_map::TempRocksDB<u64, $elem>;
            type VariantStore = map_set::TempRocksDB<deletion::Key<deletion::Dvec>, u64>;
        }
    }
}

macro_rules! deletion_rocksdb {
    ($t:ident, $elem:ty) => {
        pub type $t = ($elem, id_map::RocksDB<u64, $elem>, map_set::RocksDB<deletion::Key<deletion::Dvec>, u64>);
        impl TypeMap for $t {
            type Input = $elem;
            type Window = $elem;
            type Variant = deletion::Dvec;
            type Identifier = u64;
            type ValueStore = id_map::RocksDB<u64, $elem>;
            type VariantStore = map_set::RocksDB<deletion::Key<deletion::Dvec>, u64>;
        }
    }
}

macro_rules! substitution_echo_inmemory {
    ($t:ident, $elem:ty, $v:ty) => {
        pub type $t = ($elem, id_map::Echo<$elem>, map_set::InMemoryHash<substitution::Key<$v>, $elem>);
        impl TypeMap for $t {
            type Input = $elem;
            type Window = $v;
            type Variant = $v;
            type Identifier = $elem;
            type ValueStore = id_map::Echo<$elem>;
            type VariantStore = map_set::InMemoryHash<substitution::Key<$v>, $elem>;
        }
    }
}

macro_rules! substitution_echo_temp_rocksdb {
    ($t:ident, $elem:ty, $v:ty) => {
        pub type $t = ($elem, id_map::Echo<$elem>, map_set::TempRocksDB<substitution::Key<$v>, $elem>);
        impl TypeMap for $t {
            type Input = $elem;
            type Window = $v;
            type Variant = $v;
            type Identifier = $elem;
            type ValueStore = id_map::Echo<$elem>;
            type VariantStore = map_set::TempRocksDB<substitution::Key<$v>, $elem>;
        }
    }
}

macro_rules! substitution_echo_rocksdb {
    ($t:ident, $elem:ty, $v:ty) => {

        pub type $t = ($elem, id_map::Echo<$elem>, map_set::RocksDB<substitution::Key<$v>, $elem>);
        impl TypeMap for $t {
            type Input = $elem;
            type Window = $v;
            type Variant = $v;
            type Identifier = $elem;
            type ValueStore = id_map::Echo<$elem>;
            type VariantStore = map_set::RocksDB<substitution::Key<$v>, $elem>;
        }
    }
}

macro_rules! substitution_map_inmemory {
    ($t:ident, $elem:ty, $v:ty) => {
        pub type $t = ($elem, id_map::HashMap<u64, $elem>, map_set::InMemoryHash<substitution::Key<$v>, u64>);
        impl TypeMap for $t {
            type Input = $elem;
            type Window = $v;
            type Variant = $v;
            type Identifier = u64;
            type ValueStore = id_map::HashMap<u64, $elem>;
            type VariantStore = map_set::InMemoryHash<substitution::Key<$v>, u64>;
        }
    }
}

macro_rules! substitution_map_temp_rocksdb {
    ($t:ident, $elem:ty, $v:ty) => {
        pub type $t = ($elem, id_map::TempRocksDB<u64, $elem>, map_set::TempRocksDB<substitution::Key<$v>, u64>);
        impl TypeMap for $t {
            type Input = $elem;
            type Window = $v;
            type Variant = $v;
            type Identifier = u64;
            type ValueStore = id_map::TempRocksDB<u64, $elem>;
            type VariantStore = map_set::TempRocksDB<substitution::Key<$v>, u64>;
        }
    }
}

macro_rules! substitution_map_rocksdb {
    ($t:ident, $elem:ty, $v:ty) => {
        pub type $t = ($elem, id_map::RocksDB<u64, $elem>, map_set::RocksDB<substitution::Key<$v>, u64>);
        impl TypeMap for $t {
            type Input = $elem;
            type Window = $v;
            type Variant = $v;
            type Identifier = u64;
            type ValueStore = id_map::RocksDB<u64, $elem>;
            type VariantStore = map_set::RocksDB<substitution::Key<$v>, u64>;
        }
    }
}


deletion_inmemory!(VecU8InMemory, Vec<u8>);
deletion_inmemory!(VecU16InMemory, Vec<u16>);
deletion_inmemory!(VecU32InMemory, Vec<u32>);
deletion_inmemory!(VecU64InMemory, Vec<u64>);
deletion_inmemory!(VecU64x2InMemory, Vec<[u64; 2]>);
deletion_inmemory!(VecU64x4InMemory, Vec<[u64; 4]>);

deletion_temp_rocksdb!(VecU8TempRocksDB, Vec<u8>);
deletion_temp_rocksdb!(VecU16TempRocksDB, Vec<u16>);
deletion_temp_rocksdb!(VecU32TempRocksDB, Vec<u32>);
deletion_temp_rocksdb!(VecU64TempRocksDB, Vec<u64>);
deletion_temp_rocksdb!(VecU64x2TempRocksDB, Vec<[u64; 2]>);
deletion_temp_rocksdb!(VecU64x4TempRocksDB, Vec<[u64; 4]>);

deletion_rocksdb!(VecU8RocksDB, Vec<u8>);
deletion_rocksdb!(VecU16RocksDB, Vec<u16>);
deletion_rocksdb!(VecU32RocksDB, Vec<u32>);
deletion_rocksdb!(VecU64RocksDB, Vec<u64>);
deletion_rocksdb!(VecU64x2RocksDB, Vec<[u64; 2]>);
deletion_rocksdb!(VecU64x4RocksDB, Vec<[u64; 4]>);


substitution_echo_inmemory!(U64wU8InMemory, u64, u8);
substitution_echo_inmemory!(U64wU16InMemory, u64, u16);
substitution_echo_inmemory!(U64wU32InMemory, u64, u32);
substitution_echo_inmemory!(U64wU64InMemory, u64, u64);
substitution_echo_inmemory!(U32wU8InMemory, u32, u8);
substitution_echo_inmemory!(U32wU16InMemory, u32, u16);
substitution_echo_inmemory!(U32wU32InMemory, u32, u32);
substitution_echo_inmemory!(U16wU8InMemory, u16, u8);
substitution_echo_inmemory!(U16wU16InMemory, u16, u16);
substitution_echo_inmemory!(U8wU8InMemory, u8, u8);

substitution_echo_temp_rocksdb!(U64wU8TempRocksDB, u64, u8);
substitution_echo_temp_rocksdb!(U64wU16TempRocksDB, u64, u16);
substitution_echo_temp_rocksdb!(U64wU32TempRocksDB, u64, u32);
substitution_echo_temp_rocksdb!(U64wU64TempRocksDB, u64, u64);
substitution_echo_temp_rocksdb!(U32wU8TempRocksDB, u32, u8);
substitution_echo_temp_rocksdb!(U32wU16TempRocksDB, u32, u16);
substitution_echo_temp_rocksdb!(U32wU32TempRocksDB, u32, u32);
substitution_echo_temp_rocksdb!(U16wU8TempRocksDB, u16, u8);
substitution_echo_temp_rocksdb!(U16wU16TempRocksDB, u16, u16);
substitution_echo_temp_rocksdb!(U8wU8TempRocksDB, u8, u8);

substitution_echo_rocksdb!(U64wU8RocksDB, u64, u8);
substitution_echo_rocksdb!(U64wU16RocksDB, u64, u16);
substitution_echo_rocksdb!(U64wU32RocksDB, u64, u32);
substitution_echo_rocksdb!(U64wU64RocksDB, u64, u64);
substitution_echo_rocksdb!(U32wU8RocksDB, u32, u8);
substitution_echo_rocksdb!(U32wU16RocksDB, u32, u16);
substitution_echo_rocksdb!(U32wU32RocksDB, u32, u32);
substitution_echo_rocksdb!(U16wU8RocksDB, u16, u8);
substitution_echo_rocksdb!(U16wU16RocksDB, u16, u16);
substitution_echo_rocksdb!(U8wU8RocksDB, u8, u8);


substitution_map_inmemory!(U64x4wU8InMemory, [u64; 4], u8);
substitution_map_inmemory!(U64x4wU16InMemory, [u64; 4], u16);
substitution_map_inmemory!(U64x4wU32InMemory, [u64; 4], u32);
substitution_map_inmemory!(U64x4wU64InMemory, [u64; 4], u64);
substitution_map_inmemory!(U64x4wU64x2InMemory, [u64; 4], [u64; 2]);
substitution_map_inmemory!(U64x4wU64x4InMemory, [u64; 4], [u64; 4]);
substitution_map_inmemory!(U64x2wU8InMemory, [u64; 2], u8);
substitution_map_inmemory!(U64x2wU16InMemory, [u64; 2], u16);
substitution_map_inmemory!(U64x2wU32InMemory, [u64; 2], u32);
substitution_map_inmemory!(U64x2wU64InMemory, [u64; 2], u64);
substitution_map_inmemory!(U64x2wU64x2InMemory, [u64; 2], [u64; 2]);

substitution_map_temp_rocksdb!(U64x4wU8TempRocksDB, [u64; 4], u8);
substitution_map_temp_rocksdb!(U64x4wU16TempRocksDB, [u64; 4], u16);
substitution_map_temp_rocksdb!(U64x4wU32TempRocksDB, [u64; 4], u32);
substitution_map_temp_rocksdb!(U64x4wU64TempRocksDB, [u64; 4], u64);
substitution_map_temp_rocksdb!(U64x4wU64x2TempRocksDB, [u64; 4], [u64; 2]);
substitution_map_temp_rocksdb!(U64x4wU64x4TempRocksDB, [u64; 4], [u64; 4]);
substitution_map_temp_rocksdb!(U64x2wU8TempRocksDB, [u64; 2], u8);
substitution_map_temp_rocksdb!(U64x2wU16TempRocksDB, [u64; 2], u16);
substitution_map_temp_rocksdb!(U64x2wU32TempRocksDB, [u64; 2], u32);
substitution_map_temp_rocksdb!(U64x2wU64TempRocksDB, [u64; 2], u64);
substitution_map_temp_rocksdb!(U64x2wU64x2TempRocksDB, [u64; 2], [u64; 2]);

substitution_map_rocksdb!(U64x4wU8RocksDB, [u64; 4], u8);
substitution_map_rocksdb!(U64x4wU16RocksDB, [u64; 4], u16);
substitution_map_rocksdb!(U64x4wU32RocksDB, [u64; 4], u32);
substitution_map_rocksdb!(U64x4wU64RocksDB, [u64; 4], u64);
substitution_map_rocksdb!(U64x4wU64x2RocksDB, [u64; 4], [u64; 2]);
substitution_map_rocksdb!(U64x4wU64x4RocksDB, [u64; 4], [u64; 4]);
substitution_map_rocksdb!(U64x2wU8RocksDB, [u64; 2], u8);
substitution_map_rocksdb!(U64x2wU16RocksDB, [u64; 2], u16);
substitution_map_rocksdb!(U64x2wU32RocksDB, [u64; 2], u32);
substitution_map_rocksdb!(U64x2wU64RocksDB, [u64; 2], u64);
substitution_map_rocksdb!(U64x2wU64x2RocksDB, [u64; 2], [u64; 2]);

impl Factory for Vec<[u64; 4]> {
    fn build(dimensions: usize, tolerance: usize, backend: StorageBackend) -> Box<Database<Vec<[u64; 4]>>> {
        match backend {
            StorageBackend::InMemory => {
                let db: deletion::DB<VecU64x4InMemory> = deletion::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            StorageBackend::TempRocksDB => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: deletion::DB<VecU64x4TempRocksDB> = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            StorageBackend::RocksDB(ref path) => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: deletion::DB<VecU64x4RocksDB> = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
        }
    }
}

impl Factory for Vec<[u64; 2]> {
    fn build(dimensions: usize, tolerance: usize, backend: StorageBackend) -> Box<Database<Vec<[u64; 2]>>> {
        match backend {
            StorageBackend::InMemory => {
                let db: deletion::DB<VecU64x2InMemory> = deletion::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            StorageBackend::TempRocksDB => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: deletion::DB<VecU64x2TempRocksDB> = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            StorageBackend::RocksDB(ref path) => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: deletion::DB<VecU64x2RocksDB> = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
        }
    }
}

impl Factory for Vec<u64> {
    fn build(dimensions: usize, tolerance: usize, backend: StorageBackend) -> Box<Database<Vec<u64>>> {
        match backend {
            StorageBackend::InMemory => {
                let db: deletion::DB<VecU64InMemory> = deletion::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            StorageBackend::TempRocksDB => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: deletion::DB<VecU64TempRocksDB> = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            StorageBackend::RocksDB(ref path) => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: deletion::DB<VecU64RocksDB> = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
        }
    }
}

impl Factory for Vec<u32> {
    fn build(dimensions: usize, tolerance: usize, backend: StorageBackend) -> Box<Database<Vec<u32>>> {
        match backend {
            StorageBackend::InMemory => {
                let db: deletion::DB<VecU32InMemory> = deletion::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            StorageBackend::TempRocksDB => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: deletion::DB<VecU32TempRocksDB> = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            StorageBackend::RocksDB(ref path) => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: deletion::DB<VecU32RocksDB> = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
        }
    }
}

impl Factory for Vec<u16> {
    fn build(dimensions: usize, tolerance: usize, backend: StorageBackend) -> Box<Database<Vec<u16>>> {
        match backend {
            StorageBackend::InMemory => {
                let db: deletion::DB<VecU16InMemory> = deletion::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            StorageBackend::TempRocksDB => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: deletion::DB<VecU16TempRocksDB> = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            StorageBackend::RocksDB(ref path) => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: deletion::DB<VecU16RocksDB> = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
        }
    }
}

impl Factory for Vec<u8> {
    fn build(dimensions: usize, tolerance: usize, backend: StorageBackend) -> Box<Database<Vec<u8>>> {
        match backend {
            StorageBackend::InMemory => {
                let db: deletion::DB<VecU8InMemory> = deletion::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            StorageBackend::TempRocksDB => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: deletion::DB<VecU8TempRocksDB> = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            StorageBackend::RocksDB(ref path) => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: deletion::DB<VecU8RocksDB> = deletion::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
        }
    }
}

impl Factory for [u64; 4] {
    fn build(dimensions: usize, tolerance: usize, backend: StorageBackend) -> Box<Database<[u64; 4]>> {
        let partitions = (tolerance + 3) / 2;
        let partition_bits = Ratio::new_raw(dimensions, partitions).ceil().to_integer();

        match (partition_bits, backend) {
            (b, StorageBackend::InMemory) if b <= 8 => {
                let db: substitution::DB<U64x4wU8InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 16 => {
                let db: substitution::DB<U64x4wU16InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 32 => {
                let db: substitution::DB<U64x4wU32InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 64 => {
                let db: substitution::DB<U64x4wU64InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 128 => {
                let db: substitution::DB<U64x4wU64x2InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 256 => {
                let db: substitution::DB<U64x4wU64x2InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 8 => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64x4wU8TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 16 => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64x4wU16TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 32 => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64x4wU32TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 64 => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64x4wU64TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 128 => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64x4wU64x2TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 256 => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64x4wU64x4TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 8 => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = path.clone();
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64x4wU8RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 16 => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = path.clone();
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64x4wU16RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 32 => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64x4wU32RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 64 => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64x4wU64RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 128 => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64x4wU64x2RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 256 => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64x4wU64x4RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            _ => panic!("Unsupported tolerance"),
        }
    }
}

impl Factory for [u64; 2] {
    fn build(dimensions: usize, tolerance: usize, backend: StorageBackend) -> Box<Database<[u64; 2]>> {
        let partitions = (tolerance + 3) / 2;
        let partition_bits = Ratio::new_raw(dimensions, partitions).ceil().to_integer();

        match (partition_bits, backend) {
            (b, StorageBackend::InMemory) if b <= 8 => {
                let db: substitution::DB<U64x2wU8InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 16 => {
                let db: substitution::DB<U64x2wU16InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 32 => {
                let db: substitution::DB<U64x2wU32InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 64 => {
                let db: substitution::DB<U64x2wU64InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 128 => {
                let db: substitution::DB<U64x2wU64x2InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 8 => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64x2wU8TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 16 => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64x2wU16TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 32 => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64x2wU32TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 64 => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64x2wU64TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 128 => {
                let id_map = id_map::TempRocksDB::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64x2wU64x2TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 8 => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = path.clone();
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64x2wU8RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 16 => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = path.clone();
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64x2wU16RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 32 => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64x2wU32RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 64 => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64x2wU64RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 128 => {
                let mut id_map_path = path.clone();
                id_map_path.push("id_map");
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::RocksDB::new(id_map_path.to_str().unwrap());
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64x2wU64x2RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            _ => panic!("Unsupported tolerance"),
        }
    }
}

impl Factory for u64 {
    fn build(dimensions: usize, tolerance: usize, backend: StorageBackend) -> Box<Database<u64>> {
        let partitions = (tolerance + 3) / 2;
        let partition_bits = Ratio::new_raw(dimensions, partitions).ceil().to_integer();

        match (partition_bits, backend) {
            (b, StorageBackend::InMemory) if b <= 8 => {
                let db: substitution::DB<U64wU8InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 16 => {
                let db: substitution::DB<U64wU16InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 32 => {
                let db: substitution::DB<U64wU32InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 64 => {
                let db: substitution::DB<U64wU64InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 8 => {
                let id_map = id_map::Echo::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64wU8TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 16 => {
                let id_map = id_map::Echo::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64wU16TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 32 => {
                let id_map = id_map::Echo::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64wU32TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 64 => {
                let id_map = id_map::Echo::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U64wU64TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 8 => {
                let mut map_set_path = path.clone();
                map_set_path.push("map_set");

                let id_map = id_map::Echo::new();
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64wU8RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 16 => {
                let mut map_set_path = path.clone();
                map_set_path.push("map_set");

                let id_map = id_map::Echo::new();
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64wU16RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 32 => {
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::Echo::new();
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64wU32RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 64 => {
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::Echo::new();
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U64wU64RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            _ => panic!("Unsupported tolerance"),
        }
    }
}

impl Factory for u32 {
    fn build(dimensions: usize, tolerance: usize, backend: StorageBackend) -> Box<Database<u32>> {
        let partitions = (tolerance + 3) / 2;
        let partition_bits = Ratio::new_raw(dimensions, partitions).ceil().to_integer();

        match (partition_bits, backend) {
            (b, StorageBackend::InMemory) if b <= 8 => {
                let db: substitution::DB<U32wU8InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 16 => {
                let db: substitution::DB<U32wU16InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 32 => {
                let db: substitution::DB<U32wU32InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 8 => {
                let id_map = id_map::Echo::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U32wU8TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 16 => {
                let id_map = id_map::Echo::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U32wU16TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 32 => {
                let id_map = id_map::Echo::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U32wU32TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 8 => {
                let mut map_set_path = path.clone();
                map_set_path.push("map_set");

                let id_map = id_map::Echo::new();
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U32wU8RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 16 => {
                let mut map_set_path = path.clone();
                map_set_path.push("map_set");

                let id_map = id_map::Echo::new();
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U32wU16RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 32 => {
                let mut map_set_path = PathBuf::from(path);
                map_set_path.push("map_set");

                let id_map = id_map::Echo::new();
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U32wU32RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            _ => panic!("Unsupported tolerance"),
        }
    }
}

impl Factory for u16 {
    fn build(dimensions: usize, tolerance: usize, backend: StorageBackend) -> Box<Database<u16>> {
        let partitions = (tolerance + 3) / 2;
        let partition_bits = Ratio::new_raw(dimensions, partitions).ceil().to_integer();

        match (partition_bits, backend) {
            (b, StorageBackend::InMemory) if b <= 8 => {
                let db: substitution::DB<U16wU8InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::InMemory) if b <= 16 => {
                let db: substitution::DB<U16wU16InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 8 => {
                let id_map = id_map::Echo::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U16wU8TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 16 => {
                let id_map = id_map::Echo::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U16wU16TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 8 => {
                let mut map_set_path = path.clone();
                map_set_path.push("map_set");

                let id_map = id_map::Echo::new();
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U16wU8RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 16 => {
                let mut map_set_path = path.clone();
                map_set_path.push("map_set");

                let id_map = id_map::Echo::new();
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U16wU16RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            _ => panic!("Unsupported tolerance"),
        }
    }
}

impl Factory for u8 {
    fn build(dimensions: usize, tolerance: usize, backend: StorageBackend) -> Box<Database<u8>> {
        let partitions = (tolerance + 3) / 2;
        let partition_bits = Ratio::new_raw(dimensions, partitions).ceil().to_integer();

        match (partition_bits, backend) {
            (b, StorageBackend::InMemory) if b <= 8 => {
                let db: substitution::DB<U8wU8InMemory> = substitution::DB::new(dimensions, tolerance);
                Box::new(db)
            },
            (b, StorageBackend::TempRocksDB) if b <= 8 => {
                let id_map = id_map::Echo::new();
                let map_set = map_set::TempRocksDB::new();
                let db: substitution::DB<U8wU8TempRocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            (b, StorageBackend::RocksDB(ref path)) if b <= 8 => {
                let mut map_set_path = path.clone();
                map_set_path.push("map_set");

                let id_map = id_map::Echo::new();
                let map_set = map_set::RocksDB::new(map_set_path.to_str().unwrap());
                let db: substitution::DB<U8wU8RocksDB> = substitution::DB::with_stores(dimensions, tolerance, id_map, map_set);
                Box::new(db)
            },
            _ => panic!("Unsupported tolerance"),
        }
    }
}
