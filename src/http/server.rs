use std::io::Read;
use std::collections::HashSet;
use std::path::PathBuf;
use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, RwLock};

use iron::prelude::*;
use iron::{status, typemap};
use router::Router;
use persistent::State;
use rustc_serialize::json;

use hammer::db::id_map;
use hammer::db::map_set;
use hammer::db::Database;
use hammer::db::id_map::IDMap;
use hammer::db::map_set::{MapSet, RocksDB, TempRocksDB};
use hammer::db::substitution::{DB, Key};

use super::add;
use super::query;
use super::delete;

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: Option<PathBuf>,
    pub bind: String,
    pub bits: usize,
    pub tolerance: usize,
}

struct ConfigKey;
impl typemap::Key for ConfigKey { type Value = Config; }

struct DB64w32;
impl typemap::Key for DB64w32 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<u64, ID=u64, Window=u32, Variant=u32>>>>>; }

pub fn serve(config: Config) {
    println!("Serving with config: {:?}", config);

    let mut router = Router::new();
    router.post("/add/b64/:tolerance/:namespace", handle_add);
    router.post("/query", handle_query);
    router.post("/delete", handle_delete);

    let mut chain = Chain::new(router);
    chain.link_before(State::<ConfigKey>::one(config.clone()));
    chain.link_before(State::<DB64w32>::one(HashMap::new()));
    Iron::new(chain).http(&*config.bind).unwrap();
}

fn make_db(config: Config, bits: usize, tolerance: usize, namespace: String) -> Box<Database<u64, ID=u64, Window=u32, Variant=u32>> {
    let id_map: Box<IDMap<u64, u64>> = match config.data_dir {
        Some(ref dir) => {
            let mut value_store_path = dir.clone();
            value_store_path.push(format!("b{:03}_{:03}_{:}", bits, tolerance, namespace));
            value_store_path.push("s_var_value");

            Box::new(id_map::RocksDB::new(value_store_path.to_str().unwrap())) 
        },
        None => { Box::new(id_map::TempRocksDB::new()) },
    };

    let map_set: Box<MapSet<Key<u32>, u64>> = match config.data_dir {
        Some(ref dir) => { 
            let mut variant_store_path = dir.clone();
            variant_store_path.push(format!("b{:03}_{:03}_{:}", bits, tolerance, namespace));
            variant_store_path.push("s_var_mapset");

            Box::new(map_set::RocksDB::new(variant_store_path.to_str().unwrap())) 
        },
        None => { Box::new(map_set::TempRocksDB::new()) },
    };

    let db: Box<DB<u64, u32, u32, u64, Box<IDMap<u64, u64>>, Box<MapSet<Key<u32>, u64>>>> = Box::new(DB::with_stores(bits, tolerance, id_map, map_set));
    return db
}

fn handle_add(req: &mut Request) -> IronResult<Response> {
    let mut payload = String::new();

    match req.body.read_to_string(&mut payload) {
        Ok(_) => {},
        Err(err) => {
            return Ok(Response::with((status::BadRequest, format!("Unable to read body: {:?}", err))))
        },
    }

    match json::decode::<add::Request>(&payload) {
        Ok(req_body) => {
            let tolerance = req.extensions.get::<Router>().unwrap().find("tolerance").unwrap_or("0").parse::<usize>().unwrap();
            let namespace = req.extensions.get::<Router>().unwrap().find("namespace").unwrap_or("*").to_string();
            let dbmap_mx = req.get::<State<DB64w32>>().unwrap();

            let mut scalar_results = Vec::with_capacity(req_body.scalars.len());

            // this is a little contorted, but the idea is to optimize for the
            // frequent case where the DB being inserted into exists and only
            // incur an additional mutex lock/release when it doesn't
            let mut db_exists = true;
            loop {
                if !db_exists {
                    let config_mx = req.get::<State<ConfigKey>>().unwrap();
                    let config = config_mx.read().unwrap().clone();
                    let db = make_db(config, 64, tolerance, namespace.clone());

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

                for scalar in req_body.scalars.into_iter() {
                    let added = db.insert(scalar.clone());
                    let scalar_result = add::ScalarResult {scalar: scalar, added: added};
                    scalar_results.push(scalar_result);
                }

                break
            }

            Ok(Response::with((status::Ok, json::encode(&add::Response {scalars: scalar_results}).unwrap())))
        },

        Err(err) => {
            Ok(Response::with((status::BadRequest, format!("Unable to parse JSON: {:?}", err))))
        },
    }
}

fn handle_query(req: &mut Request) -> IronResult<Response> {
    let mut payload = String::new();

    match req.body.read_to_string(&mut payload) {
        Ok(_) => {},
        Err(err) => {
            return Ok(Response::with((status::BadRequest, format!("Unable to read body: {:?}", err))))
        },
    }

    match json::decode::<query::Request>(&payload) {
        Ok(req_body) => {
            let tolerance = req.extensions.get::<Router>().unwrap().find("tolerance").unwrap_or("0").parse::<usize>().unwrap();
            let namespace = req.extensions.get::<Router>().unwrap().find("namespace").unwrap_or("*").to_string();
            let dbmap_mx = req.get::<State<DB64w32>>().unwrap();

            let mut results = BTreeMap::new();

            match { dbmap_mx.read().unwrap().get(&(tolerance.clone(), namespace.clone())) } {
                None => {},
                Some(db_mx) => {
                    let db = db_mx.read().unwrap();

                    for scalar in req_body.scalars.into_iter() {
                        match db.get(&scalar) {
                            Some(found) => {
                                results.insert(scalar, found);
                            },
                            None => {
                                results.insert(scalar, HashSet::new());
                            },
                        }
                    }
                }
            }


            Ok(Response::with((status::Ok, json::encode(&results).unwrap())))
        },

        Err(err) => {
            Ok(Response::with((status::BadRequest, format!("Unable to parse JSON: {:?}", err))))
        },
    }
}

fn handle_delete(req: &mut Request) -> IronResult<Response> {
    let mut payload = String::new();

    match req.body.read_to_string(&mut payload) {
        Ok(_) => {},
        Err(err) => {
            return Ok(Response::with((status::BadRequest, format!("Unable to read body: {:?}", err))))
        },
    }

    match json::decode::<delete::Request>(&payload) {
        Ok(req_body) => {
            let tolerance = req.extensions.get::<Router>().unwrap().find("tolerance").unwrap_or("0").parse::<usize>().unwrap();
            let namespace = req.extensions.get::<Router>().unwrap().find("namespace").unwrap_or("*").to_string();
            let dbmap_mx = req.get::<State<DB64w32>>().unwrap();

            let mut results = BTreeMap::new();

            match { dbmap_mx.read().unwrap().get(&(tolerance.clone(), namespace.clone())) } {
                None => {
                    for scalar in req_body.scalars.into_iter() {
                        results.insert(scalar, false);
                    }
                },
                Some(db_mx) => {
                    let mut db = db_mx.write().unwrap();

                    for scalar in req_body.scalars.into_iter() {
                        results.insert(scalar, db.remove(&scalar));
                    }
                }
            }

            Ok(Response::with((status::Ok, json::encode(&results).unwrap())))
        },

        Err(err) => {
            Ok(Response::with((status::BadRequest, format!("Unable to parse JSON: {:?}", err))))
        },
    }
}
