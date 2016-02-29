use std::cmp::Eq;
use std::clone::Clone;
use std::hash::Hash;
use std::io::Read;
use std::mem::size_of;
use std::path::PathBuf;
use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, RwLock};

use bincode;
use iron::prelude::*;
use iron::{status, typemap};
use router::Router;
use persistent::State;
use rustc_serialize::json;
use rustc_serialize::base64;
use rustc_serialize::base64::{FromBase64, ToBase64};
use rustc_serialize::{Encodable, Decodable};

use hammer::db::id_map;
use hammer::db::map_set;
use hammer::db::Database;
use hammer::db::hamming::Hamming;
use hammer::db::window::Windowable;
use hammer::db::id_map::{ToID, IDMap};
use hammer::db::map_set::{MapSet, RocksDB, TempRocksDB};
use hammer::db::substitution::{DB, Key, SubstitutionVariant};

const BASE64_CONFIG: base64::Config = base64::Config{
    char_set: base64::CharacterSet::Standard,
    newline: base64::Newline::CRLF,
    pad: true,
    line_length: None,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: Option<PathBuf>,
    pub bind: String,
    pub bits: usize,
    pub tolerance: usize,
}

struct ConfigKey;
impl typemap::Key for ConfigKey { type Value = Config; }

struct B64w64;
impl typemap::Key for B64w64 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<u64, ID=u64, Window=u64, Variant=u64>>>>>; }
struct B64w32;
impl typemap::Key for B64w32 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<u64, ID=u64, Window=u32, Variant=u32>>>>>; }
struct B64w16;
impl typemap::Key for B64w16 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<u64, ID=u64, Window=u16, Variant=u16>>>>>; }
struct B64w8;
impl typemap::Key for B64w8 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<u64, ID=u64, Window=u8, Variant=u8>>>>>; }

// NOTE: Need to implement hamming for all these, and then window + hamming
// for things windowing to more than 64 bits
struct B128w128;
impl typemap::Key for B128w128 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<[u64; 2], ID=u64, Window=[u64; 2], Variant=[u64; 2]>>>>>; }
struct B128w64;
impl typemap::Key for B128w64 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<[u64; 2], ID=u64, Window=u64, Variant=u64>>>>>; }
struct B128w32;
impl typemap::Key for B128w32 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<[u64; 2], ID=u64, Window=u32, Variant=u32>>>>>; }
struct B128w16;
impl typemap::Key for B128w16 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<[u64; 2], ID=u64, Window=u16, Variant=u16>>>>>; }
struct B128w8;
impl typemap::Key for B128w8 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<[u64; 2], ID=u64, Window=u8, Variant=u8>>>>>; }

struct B256w256;
impl typemap::Key for B256w256 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<[u64; 4], ID=u64, Window=[u64; 4], Variant=[u64; 4]>>>>>; }
struct B256w128;
impl typemap::Key for B256w128 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<[u64; 4], ID=u64, Window=[u64; 2], Variant=[u64; 2]>>>>>; }
struct B256w64;
impl typemap::Key for B256w64 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<[u64; 4], ID=u64, Window=u64, Variant=u64>>>>>; }
struct B256w32;
impl typemap::Key for B256w32 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<[u64; 4], ID=u64, Window=u32, Variant=u32>>>>>; }
struct B256w16;
impl typemap::Key for B256w16 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<[u64; 4], ID=u64, Window=u16, Variant=u16>>>>>; }
struct B256w8;
impl typemap::Key for B256w8 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<[u64; 4], ID=u64, Window=u8, Variant=u8>>>>>; }

pub fn serve(config: Config) {
    println!("Serving with config: {:?}", config);

    let mut router = Router::new();
    router.post("/add/b:bits/:tolerance/:namespace", handle_add);
    router.post("/query/b64/:tolerance/:namespace", handle_query);
    router.post("/delete/b64/:tolerance/:namespace", handle_delete);

    let mut chain = Chain::new(router);
    chain.link_before(State::<ConfigKey>::one(config.clone()));

    chain.link_before(State::<B256w64>::one(HashMap::new()));
    chain.link_before(State::<B256w32>::one(HashMap::new()));
    chain.link_before(State::<B256w16>::one(HashMap::new()));
    chain.link_before(State::<B256w8>::one(HashMap::new()));

    chain.link_before(State::<B128w128>::one(HashMap::new()));
    chain.link_before(State::<B128w64>::one(HashMap::new()));
    chain.link_before(State::<B128w32>::one(HashMap::new()));
    chain.link_before(State::<B128w16>::one(HashMap::new()));
    chain.link_before(State::<B128w8>::one(HashMap::new()));

    chain.link_before(State::<B64w64>::one(HashMap::new()));
    chain.link_before(State::<B64w32>::one(HashMap::new()));
    chain.link_before(State::<B64w16>::one(HashMap::new()));
    chain.link_before(State::<B64w8>::one(HashMap::new()));

    Iron::new(chain).http(&*config.bind).unwrap();
}

fn decode_body<T>(req: &mut Request) -> Result<T, IronError> where
T: Decodable
{
    let mut payload = String::new();
    itry!(req.body.read_to_string(&mut payload));

    match json::decode::<T>(&payload) {
        Ok(req_body) => {
            Ok(req_body)
        },
        Err(err) => {
            Err(IronError::new(err, (status::BadRequest, "Unable to parse JSON")))
        }
    }
}

fn handle_add(req: &mut Request) -> IronResult<Response> {
    let req_body = try!(decode_body::<Vec<String>>(req));

    let bits = match req.extensions.get::<Router>().unwrap().find("bits") {
        Some(v) => v.parse::<usize>().unwrap(),
        None => return Ok(Response::with((status::BadRequest, "DB bitsize is required"))),
    };

    let tolerance = match req.extensions.get::<Router>().unwrap().find("tolerance") {
        Some(v) => v.parse::<usize>().unwrap(),
        None => return Ok(Response::with((status::BadRequest, "DB tolerance is required"))),
    };

    let namespace = match req.extensions.get::<Router>().unwrap().find("namespace") {
        Some(v) => v.to_string(),
        None => return Ok(Response::with((status::BadRequest, "DB namespace is required"))),
    };

    let config_mx = req.get::<State<ConfigKey>>().unwrap();

    // The number of partitions `K` which will guarantee at least one partition with a 
    // hamming distance of 1, given a query with hamming distance `D` is 
    // `K = floor((D+3)/2)`.
    //
    // This can be used to solve for an appropriate partition count and type 
    //
    // Tolerance  | Req. Part. | W   (64) | W   (128) | W   (256)
    // -----------+------------+----------+-----------+----------
    // 0          | 1          | 64       | 128       | 256
    // 1          | 2          | 32       | 64        | 128
    // 2          | 2          | 32       | 64        | 128
    // 3          | 3          | 32       | 64        | 128
    // 4          | 3          | 32       | 64        | 128
    // 5          | 4          | 16       | 32        | 64 
    // 6          | 4          | 16       | 32        | 64 
    // 7          | 5          | 16       | 32        | 64 
    // 8          | 5          | 16       | 32        | 64 
    // 9          | 6          | 16       | 32        | 64 
    // 10         | 6          | 16       | 32        | 64 
    // 11         | 7          | 16       | 32        | 64 
    // 12         | 7          | 16       | 32        | 64 
    // ...        |            |          |           |
    // 13         | 8          | 8        | 16        | 32 
    // ...        |            |          |           |
    // 29         | 16         | -        | 8         | 16
    // ...        |            |          |           |
    // 61         | 32         | -        | -         | 8
    match (bits, tolerance) {
        (64, t) if t == 0 => {
            let dbmap_mx = req.get::<State<B64w64>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (128, t) if t == 0 => {
            let dbmap_mx = req.get::<State<B128w128>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (256, t) if t == 0 => {
            let dbmap_mx = req.get::<State<B256w256>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (64, t) if 0 < t && t <= 4 => {
            let dbmap_mx = req.get::<State<B64w32>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (128, t) if 0 < t && t <= 4 => {
            let dbmap_mx = req.get::<State<B128w64>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (256, t) if 0 < t && t <= 4 => {
            let dbmap_mx = req.get::<State<B256w128>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (64, t) if 4 < t && t <= 12 => {
            let dbmap_mx = req.get::<State<B64w16>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (128, t) if 4 < t && t <= 12 => {
            let dbmap_mx = req.get::<State<B128w32>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (256, t) if 4 < t && t <= 12 => {
            let dbmap_mx = req.get::<State<B256w64>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (64, t) if 12 < t && t <= 28 => {
            let dbmap_mx = req.get::<State<B64w8>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (128, t) if 12 < t && t <= 28 => {
            let dbmap_mx = req.get::<State<B128w16>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (256, t) if 12 < t && t <= 28 => {
            let dbmap_mx = req.get::<State<B256w32>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (128, t) if 28 < t && t <= 60 => {
            let dbmap_mx = req.get::<State<B128w8>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (256, t) if 28 < t && t <= 60 => {
            let dbmap_mx = req.get::<State<B256w16>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (64, _) => {
            let dbmap_mx = req.get::<State<B64w8>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (128, _) => {
            let dbmap_mx = req.get::<State<B128w8>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        (256, _) => {
            let dbmap_mx = req.get::<State<B256w8>>().unwrap();
            add(req_body, tolerance, namespace, config_mx, dbmap_mx)
        },
        _ => Ok(Response::with((status::BadRequest, "Unsuported bitsize or tolerance"))),
    }
}

fn add<T, V>(req_body: Vec<String>, tolerance: usize, namespace: String, config_mx: Arc<RwLock<Config>>, dbmap_mx: Arc<RwLock<HashMap<(usize, String), Arc<RwLock<Box<Database<T, ID=u64, Window=V, Variant=V>>>>>>>) -> IronResult<Response>  where
T: 'static + Sync + Send + Clone + Eq + Hash + Encodable + Decodable + Hamming + Windowable<V> +ToID<u64>,
V: 'static + Sync + Send + Clone + Eq + Hash + Encodable + Decodable + SubstitutionVariant<V>,
{
    let mut results = BTreeMap::new();

    // this is a little contorted, but the idea is to optimize for the
    // frequent case where the DB being inserted into exists and only
    // incur an additional mutex lock/release when it doesn't
    let mut db_exists = true;
    loop {
        if !db_exists {
            let config = config_mx.read().unwrap().clone();
            let db: Box<Database<T, ID=u64, Window=V, Variant=V>> = make_db(config, 8 * size_of::<T>(), tolerance, namespace.clone());

            let mut dbmap = dbmap_mx.write().unwrap();
            dbmap.insert((tolerance.clone(), namespace.clone()), Arc::new(RwLock::new(db)));
        }

        let dbmap = dbmap_mx.read().unwrap();

        if !dbmap.contains_key(&(tolerance, namespace.clone())) {
            db_exists = false;
            continue
        }

        let db_mx = dbmap.get(&(tolerance, namespace)).unwrap();
        let mut db = db_mx.write().unwrap();

        for value_b64 in req_body.into_iter() {
            let value_bytes = match value_b64.from_base64() {
                Ok(v) => v,
                Err(e) => { return Ok(Response::with((status::BadRequest, format!("unable to decode base-64: {:?}", e)))) },
            };

            let value: T = match bincode::rustc_serialize::decode(&value_bytes) {
                Ok(v) => v,
                Err(e) => { return Ok(Response::with((status::BadRequest, format!("unable to decode {} bytes: {:?}", value_bytes.len(), e)))) },
            };

            results.insert(value_b64, db.insert(value.clone()));
        }

        break
    }

    let response_body = itry!(json::encode(&results));
    Ok(Response::with((status::Ok, response_body)))
}

fn handle_query(req: &mut Request) -> IronResult<Response> {
    let req_body = try!(decode_body::<Vec<String>>(req));
    let tolerance = req.extensions.get::<Router>().unwrap().find("tolerance").unwrap_or("0").parse::<usize>().unwrap();
    let namespace = req.extensions.get::<Router>().unwrap().find("namespace").unwrap_or("*").to_string();
    let dbmap_mx = req.get::<State<B64w32>>().unwrap();

    let mut results = BTreeMap::new();

    match { dbmap_mx.read().unwrap().get(&(tolerance.clone(), namespace.clone())) } {
        None => {},
        Some(db_mx) => {
            let db = db_mx.read().unwrap();

            for value_b64 in req_body.into_iter() {
                let value_bytes = itry!(value_b64.from_base64());
                let value: u64 = itry!(bincode::rustc_serialize::decode(&value_bytes));

                match db.get(&value) {
                    Some(found) => {
                        let found_b64s = found.iter().map(|v| {
                            let found_bytes = bincode::rustc_serialize::encode(v, bincode::SizeLimit::Infinite).unwrap();

                            found_bytes.to_base64(BASE64_CONFIG)
                        }).collect();

                        results.insert(value_b64, found_b64s);
                    },
                    None => {
                        results.insert(value_b64, Vec::new());
                    },
                }
            }
        }
    }

    let response_body = itry!(json::encode(&results));
    Ok(Response::with((status::Ok, response_body)))
}

fn handle_delete(req: &mut Request) -> IronResult<Response> {
    let req_body = try!(decode_body::<Vec<String>>(req));
    let tolerance = req.extensions.get::<Router>().unwrap().find("tolerance").unwrap_or("0").parse::<usize>().unwrap();
    let namespace = req.extensions.get::<Router>().unwrap().find("namespace").unwrap_or("*").to_string();
    let dbmap_mx = req.get::<State<B64w32>>().unwrap();

    let mut results = BTreeMap::new();

    match { dbmap_mx.read().unwrap().get(&(tolerance.clone(), namespace.clone())) } {
        None => {
            for value in req_body.into_iter() {
                results.insert(value, false);
            }
        },
        Some(db_mx) => {
            let mut db = db_mx.write().unwrap();

            for value_b64 in req_body.into_iter() {
                let value_bytes = itry!(value_b64.from_base64());
                let value: u64 = itry!(bincode::rustc_serialize::decode(&value_bytes));

                results.insert(value_b64, db.remove(&value));
            }
        }
    }

    let response_body = itry!(json::encode(&results));
    Ok(Response::with((status::Ok, response_body)))
}

fn make_db<T, V>(config: Config, bits: usize, tolerance: usize, namespace: String) -> Box<Database<T, ID=u64, Window=V, Variant=V>> where 
T: 'static + Sync + Send + Clone + Eq + Hash + Encodable + Decodable + Hamming + Windowable<V> +ToID<u64>,
V: 'static + Sync + Send + Clone + Eq + Hash + Encodable + Decodable + SubstitutionVariant<V>,
{
    let id_map: Box<IDMap<u64, T>> = match config.data_dir {
        Some(ref dir) => {
            let mut value_store_path = dir.clone();
            value_store_path.push(format!("b{:03}_{:03}_{:}", bits, tolerance, namespace));
            value_store_path.push("s_var_value");

            Box::new(id_map::RocksDB::new(value_store_path.to_str().unwrap())) 
        },
        None => { Box::new(id_map::TempRocksDB::new()) },
    };

    let map_set: Box<MapSet<Key<V>, u64>> = match config.data_dir {
        Some(ref dir) => { 
            let mut variant_store_path = dir.clone();
            variant_store_path.push(format!("b{:03}_{:03}_{:}", bits, tolerance, namespace));
            variant_store_path.push("s_var_mapset");

            Box::new(map_set::RocksDB::new(variant_store_path.to_str().unwrap())) 
        },
        None => { Box::new(map_set::TempRocksDB::new()) },
    };

    let db: Box<DB<T, V, V, u64, Box<IDMap<u64, T>>, Box<MapSet<Key<V>, u64>>>> = Box::new(DB::with_stores(bits, tolerance, id_map, map_set));
    return db
}
