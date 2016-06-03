//! TypeMap implementations for common types

use db::id_map;
use db::map_set;
use db::deletion;
use db::substitution;
use db::TypeMap;

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
