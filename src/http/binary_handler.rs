use std::clone::Clone;
use std::hash::Hash;
use std::cmp::Eq;
use std::io::Read;
use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, RwLock};

use bincode;
use iron::prelude::*;
use iron::status;
use router::Router;
use persistent::State;
use rustc_serialize::json;
use rustc_serialize::base64::{FromBase64, ToBase64};
use rustc_serialize::{Encodable, Decodable};

use hammer::db::{BinaryDB, StorageBackend};
use hammer::db::Database;
use hammer::db::id_map::IDMap;
use hammer::db::map_set::MapSet;

use http::{Config, ConfigKey, B32, B64, B128, B256, decode_body, BASE64_CONFIG};

pub fn add(req: &mut Request) -> IronResult<Response> {
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

    match bits {
        32 => {
            let dbmap_mx = req.get::<State<B32>>().unwrap();
            do_add(req_body, bits, tolerance, namespace, config_mx, dbmap_mx)
        },
        64 => {
            let dbmap_mx = req.get::<State<B64>>().unwrap();
            do_add(req_body, bits, tolerance, namespace, config_mx, dbmap_mx)
        },
        128 => {
            let dbmap_mx = req.get::<State<B128>>().unwrap();
            do_add(req_body, bits, tolerance, namespace, config_mx, dbmap_mx)
        },
        256 => {
            let dbmap_mx = req.get::<State<B256>>().unwrap();
            do_add(req_body, bits, tolerance, namespace, config_mx, dbmap_mx)
        },
        _ => Ok(Response::with((status::BadRequest, "Unsuported bitsize"))),
    }
}

fn do_add<T>(req_body: Vec<String>, bits: usize, tolerance: usize, namespace: String, config_mx: Arc<RwLock<Config>>, dbmap_mx: Arc<RwLock<HashMap<(usize, String), Arc<RwLock<Box<Database<T>>>>>>>) -> IronResult<Response> where
T: Clone + BinaryDB + Decodable,
{
    let mut results = BTreeMap::new();

    // this is a little contorted, but the idea is to optimize for the
    // frequent case where the DB being inserted into exists and only
    // incur an additional mutex lock/release when it doesn't
    let mut db_exists = true;
    loop {
        if !db_exists {
            let config = {
                config_mx.read().unwrap().clone()
            };

            let backend = match config.data_dir {
                Some(ref dir) => {
                    let mut value_store_path = dir.clone();
                    value_store_path.push(format!("b{:03}_{:03}_{:}", bits, tolerance, namespace));

                    StorageBackend::RocksDB(value_store_path.to_str().unwrap().to_string())
                },
                None => StorageBackend::InMemory
            };

            let db = BinaryDB::new(tolerance, backend);

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

pub fn query(req: &mut Request) -> IronResult<Response> {
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

    match bits {
        32 => {
            let dbmap_mx = req.get::<State<B32>>().unwrap();
            do_query(req_body, tolerance, namespace, dbmap_mx)
        },
        64 => {
            let dbmap_mx = req.get::<State<B64>>().unwrap();
            do_query(req_body, tolerance, namespace, dbmap_mx)
        },
        128 => {
            let dbmap_mx = req.get::<State<B128>>().unwrap();
            do_query(req_body, tolerance, namespace, dbmap_mx)
        },
        256 => {
            let dbmap_mx = req.get::<State<B256>>().unwrap();
            do_query(req_body, tolerance, namespace, dbmap_mx)
        },
        _ => Ok(Response::with((status::BadRequest, "Unsuported bitsize"))),
    }
}

fn do_query<T>(req_body: Vec<String>, tolerance: usize, namespace: String, dbmap_mx: Arc<RwLock<HashMap<(usize, String), Arc<RwLock<Box<Database<T>>>>>>>) -> IronResult<Response> where
T: Eq + Hash + Clone + BinaryDB + Encodable + Decodable,
{
    let mut results = BTreeMap::new();

    match { dbmap_mx.read().unwrap().get(&(tolerance.clone(), namespace.clone())) } {
        None => {},
        Some(db_mx) => {
            let db = db_mx.read().unwrap();

            for value_b64 in req_body.into_iter() {
                let value_bytes = itry!(value_b64.from_base64());
                let value: T = itry!(bincode::rustc_serialize::decode(&value_bytes));

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

pub fn delete(req: &mut Request) -> IronResult<Response> {
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

    match bits {
        32 => {
            let dbmap_mx = req.get::<State<B32>>().unwrap();
            do_delete(req_body, tolerance, namespace, dbmap_mx)
        },
        64 => {
            let dbmap_mx = req.get::<State<B64>>().unwrap();
            do_delete(req_body, tolerance, namespace, dbmap_mx)
        },
        128 => {
            let dbmap_mx = req.get::<State<B128>>().unwrap();
            do_delete(req_body, tolerance, namespace, dbmap_mx)
        },
        256 => {
            let dbmap_mx = req.get::<State<B256>>().unwrap();
            do_delete(req_body, tolerance, namespace, dbmap_mx)
        },
        _ => Ok(Response::with((status::BadRequest, "Unsuported bitsize or tolerance"))),
    }
}

fn do_delete<T>(req_body: Vec<String>, tolerance: usize, namespace: String, dbmap_mx: Arc<RwLock<HashMap<(usize, String), Arc<RwLock<Box<Database<T>>>>>>>) -> IronResult<Response> where
T: Eq + Hash + Clone + BinaryDB + Encodable + Decodable,
{
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
                let value: T = itry!(bincode::rustc_serialize::decode(&value_bytes));

                results.insert(value_b64, db.remove(&value));
            }
        }
    }

    let response_body = itry!(json::encode(&results));
    Ok(Response::with((status::Ok, response_body)))
}
