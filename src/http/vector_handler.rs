use std::clone::Clone;
use std::hash::Hash;
use std::cmp::Eq;
use std::io::Read;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use bincode;
use iron::prelude::*;
use iron::status;
use router::Router;
use persistent::State;
use rustc_serialize::json;
use rustc_serialize::base64::{FromBase64, ToBase64};
use rustc_serialize::{Encodable, Decodable};
use rustc_serialize::json::ToJson;

use hammer::db::{Database, Factory, StorageBackend};
use hammer::db::id_map::IDMap;
use hammer::db::map_set::MapSet;
use hammer::db::typemap::*;

use http::{Config, ConfigKey, V32, V64, V128, V256, decode_body, BASE64_CONFIG, AddResult, QueryResult, DeleteResult};

pub fn add(req: &mut Request) -> IronResult<Response> {
    let req_body = try!(decode_body::<Vec<Vec<String>>>(req));

    let bits = match req.extensions.get::<Router>().unwrap().find("bits") {
        Some(v) => v.parse::<usize>().unwrap(),
        None => return Ok(Response::with((status::BadRequest, "DB bitsize is required"))),
    };

    let dimensions = match req.extensions.get::<Router>().unwrap().find("dimensions") {
        Some(v) => v.parse::<usize>().unwrap(),
        None => return Ok(Response::with((status::BadRequest, "DB dimensions is required"))),
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
            let dbmap_mx = req.get::<State<V32>>().unwrap();
            do_add(req_body, bits, dimensions, tolerance, namespace, config_mx, dbmap_mx)
        },
        64 => {
            let dbmap_mx = req.get::<State<V64>>().unwrap();
            do_add(req_body, bits, dimensions, tolerance, namespace, config_mx, dbmap_mx)
        },
        128 => {
            let dbmap_mx = req.get::<State<V128>>().unwrap();
            do_add(req_body, bits, dimensions, tolerance, namespace, config_mx, dbmap_mx)
        },
        256 => {
            let dbmap_mx = req.get::<State<V256>>().unwrap();
            do_add(req_body, bits, dimensions, tolerance, namespace, config_mx, dbmap_mx)
        },
        _ => Ok(Response::with((status::BadRequest, "Unsuported bitsize"))),
    }
}

fn do_add<T>(req_body: Vec<Vec<String>>, bits: usize, dimensions: usize, tolerance: usize, namespace: String, config_mx: Arc<RwLock<Config>>, dbmap_mx: Arc<RwLock<HashMap<(usize, usize, String), Arc<RwLock<Box<Database<Vec<T>>>>>>>>) -> IronResult<Response> where
T: Clone + Decodable,
Vec<T>: Factory,
{
    let mut results = Vec::with_capacity(req_body.len());

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
                    value_store_path.push(format!("v{:03}_{:03}_{:03}_{:}", bits, dimensions, tolerance, namespace));

                    StorageBackend::RocksDB(value_store_path)
                },
                None => StorageBackend::InMemory
            };

            let db = Factory::build(dimensions, tolerance, backend);

            let mut dbmap = dbmap_mx.write().unwrap();
            // NOTE: Need to verify this key wasn't inserted earlier and we lost a race
            dbmap.insert((dimensions.clone(), tolerance.clone(), namespace.clone()), Arc::new(RwLock::new(db)));
        }

        let dbmap = dbmap_mx.read().unwrap();

        if !dbmap.contains_key(&(dimensions, tolerance, namespace.clone())) {
            db_exists = false;
            continue
        }

        let db_mx = dbmap.get(&(dimensions, tolerance, namespace)).unwrap();
        let mut db = db_mx.write().unwrap();

        'vector: for vector_b64 in req_body.into_iter() {
            let mut vector = Vec::with_capacity(dimensions);

            for item_b64 in vector_b64.into_iter() {
                let item_bytes = match item_b64.from_base64() {
                    Ok(v) => v,
                    Err(e) => {
                        results.push(AddResult::Err(format!("unable to base64-decode '{}': {:?}", item_b64, e)));
                        continue 'vector;
                    }
                };

                let item: T = match bincode::rustc_serialize::decode(&item_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        results.push(AddResult::Err(format!("unable to decode '{}': {:?}", item_b64, e)));
                        continue 'vector;
                    },
                };

                vector.push(item);
            }

            if vector.len() != dimensions {
                results.push(AddResult::Err(format!("expected vector length to be {}, not {}", dimensions, vector.len())));
                continue 'vector;
            }

            match db.insert(vector) {
                true => { results.push(AddResult::Ok); },
                false => { results.push(AddResult::Exists); },
            }
        }

        break
    }

    let response_body = json::encode(&results.to_json()).unwrap();
    Ok(Response::with((status::Ok, response_body)))
}

pub fn query(req: &mut Request) -> IronResult<Response> {
    let req_body = try!(decode_body::<Vec<Vec<String>>>(req));

    let bits = match req.extensions.get::<Router>().unwrap().find("bits") {
        Some(v) => v.parse::<usize>().unwrap(),
        None => return Ok(Response::with((status::BadRequest, "DB bitsize is required"))),
    };

    let dimensions = match req.extensions.get::<Router>().unwrap().find("dimensions") {
        Some(v) => v.parse::<usize>().unwrap(),
        None => return Ok(Response::with((status::BadRequest, "DB dimensions is required"))),
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
            let dbmap_mx = req.get::<State<V32>>().unwrap();
            do_query(req_body, dimensions, tolerance, namespace, dbmap_mx)
        },
        64 => {
            let dbmap_mx = req.get::<State<V64>>().unwrap();
            do_query(req_body, dimensions, tolerance, namespace, dbmap_mx)
        },
        128 => {
            let dbmap_mx = req.get::<State<V128>>().unwrap();
            do_query(req_body, dimensions, tolerance, namespace, dbmap_mx)
        },
        256 => {
            let dbmap_mx = req.get::<State<V256>>().unwrap();
            do_query(req_body, dimensions, tolerance, namespace, dbmap_mx)
        },
        _ => Ok(Response::with((status::BadRequest, "Unsuported bitsize"))),
    }
}

fn do_query<T>(req_body: Vec<Vec<String>>, dimensions: usize, tolerance: usize, namespace: String, dbmap_mx: Arc<RwLock<HashMap<(usize, usize, String), Arc<RwLock<Box<Database<Vec<T>>>>>>>>) -> IronResult<Response> where
T: Eq + Hash + Clone + Encodable + Decodable,
{
    let mut results = Vec::with_capacity(req_body.len());

    match { dbmap_mx.read().unwrap().get(&(dimensions.clone(), tolerance.clone(), namespace.clone())) } {
        None => {
            for _ in 0..req_body.len() {
                results.push(QueryResult::None);
            }
        },
        Some(db_mx) => {
            let db = db_mx.read().unwrap();

            'vector: for vector_b64 in req_body.into_iter() {
                let mut vector = Vec::with_capacity(dimensions);

                for item_b64 in vector_b64.into_iter() {
                    let item_bytes = match item_b64.from_base64() {
                        Ok(v) => v,
                        Err(e) => {
                            results.push(QueryResult::Err(format!("unable to base64-decode '{}': {:?}", item_b64, e)));
                            continue 'vector;
                        }
                    };

                    let item: T = match bincode::rustc_serialize::decode(&item_bytes) {
                        Ok(v) => v,
                        Err(e) => {
                            results.push(QueryResult::Err(format!("unable to decode '{}': {:?}", item_b64, e)));
                            continue 'vector;
                        },
                    };

                    vector.push(item);
                }

                if vector.len() != dimensions {
                    results.push(QueryResult::Err(format!("expected vector length to be {}, not {}", dimensions, vector.len())));
                    continue 'vector;
                }

                match db.get(&vector) {
                    Some(found) => {
                        let found_b64s: Vec<Vec<String>> = found.iter().map(|v| {
                            v.iter().map(|item| {
                                let found_bytes = bincode::rustc_serialize::encode(item, bincode::SizeLimit::Infinite).unwrap();

                                found_bytes.to_base64(BASE64_CONFIG)
                            }).collect()
                        }).collect();

                        results.push(QueryResult::Ok(found_b64s));
                    },
                    None => {
                        results.push(QueryResult::None);
                    },
                }
            }
        }
    }

    let response_body = json::encode(&results.to_json()).unwrap();
    Ok(Response::with((status::Ok, response_body)))
}

pub fn delete(req: &mut Request) -> IronResult<Response> {
    let req_body = try!(decode_body::<Vec<Vec<String>>>(req));

    let bits = match req.extensions.get::<Router>().unwrap().find("bits") {
        Some(v) => v.parse::<usize>().unwrap(),
        None => return Ok(Response::with((status::BadRequest, "DB bitsize is required"))),
    };

    let dimensions = match req.extensions.get::<Router>().unwrap().find("dimensions") {
        Some(v) => v.parse::<usize>().unwrap(),
        None => return Ok(Response::with((status::BadRequest, "DB dimensions is required"))),
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
            let dbmap_mx = req.get::<State<V32>>().unwrap();
            do_delete(req_body, dimensions, tolerance, namespace, dbmap_mx)
        },
        64 => {
            let dbmap_mx = req.get::<State<V64>>().unwrap();
            do_delete(req_body, dimensions, tolerance, namespace, dbmap_mx)
        },
        128 => {
            let dbmap_mx = req.get::<State<V128>>().unwrap();
            do_delete(req_body, dimensions, tolerance, namespace, dbmap_mx)
        },
        256 => {
            let dbmap_mx = req.get::<State<V256>>().unwrap();
            do_delete(req_body, dimensions, tolerance, namespace, dbmap_mx)
        },
        _ => Ok(Response::with((status::BadRequest, "Unsuported bitsize or tolerance"))),
    }
}

fn do_delete<T>(req_body: Vec<Vec<String>>, dimensions: usize, tolerance: usize, namespace: String, dbmap_mx: Arc<RwLock<HashMap<(usize, usize, String), Arc<RwLock<Box<Database<Vec<T>>>>>>>>) -> IronResult<Response> where
T: Eq + Hash + Clone + Encodable + Decodable,
{
    let mut results = Vec::with_capacity(req_body.len());

    match { dbmap_mx.read().unwrap().get(&(dimensions.clone(), tolerance.clone(), namespace.clone())) } {
        None => {
            for _ in 0..req_body.len() {
                results.push(DeleteResult::NotFound);
            }
        },
        Some(db_mx) => {
            let mut db = db_mx.write().unwrap();

            'vector: for vector_b64 in req_body.into_iter() {
                let mut vector = Vec::with_capacity(dimensions);

                for item_b64 in vector_b64.into_iter() {
                    let item_bytes = match item_b64.from_base64() {
                        Ok(v) => v,
                        Err(e) => {
                            results.push(DeleteResult::Err(format!("unable to base64-decode '{}': {:?}", item_b64, e)));
                            continue 'vector;
                        }
                    };

                    let item: T = match bincode::rustc_serialize::decode(&item_bytes) {
                        Ok(v) => v,
                        Err(e) => {
                            results.push(DeleteResult::Err(format!("unable to decode '{}': {:?}", item_b64, e)));
                            continue 'vector;
                        },
                    };

                    vector.push(item);
                }

                if vector.len() != dimensions {
                    results.push(DeleteResult::Err(format!("expected vector length to be {}, not {}", dimensions, vector.len())));
                    continue 'vector;
                }
                match db.remove(&vector) {
                    true => { results.push(DeleteResult::Ok); },
                    false => { results.push(DeleteResult::NotFound); },
                }
            }
        }
    }

    let response_body = json::encode(&results.to_json()).unwrap();
    Ok(Response::with((status::Ok, response_body)))
}
