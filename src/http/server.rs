use std::io::Read;
use std::collections::HashSet;
use std::path::PathBuf;

use iron::prelude::*;
use iron::{status, typemap};
use router::Router;
use persistent::State;
use rustc_serialize::json;

use hammer::db::Database;
use hammer::db::id_map::{IDMap, Echo};
use hammer::db::map_set::{MapSet, RocksDB, TempRocksDB};
use hammer::db::substitution::{DB, Key};

use super::add;
use super::query;
use super::delete;

#[derive(Debug)]
pub struct Server {
    pub data_dir: Option<PathBuf>,
    pub bind: String,
    pub bits: usize,
    pub tolerance: usize,
}

struct DBKey;
impl typemap::Key for DBKey { type Value = Box<Database<Value=u64>>; }

impl Server {
    pub fn serve(self) {
        println!("Serving with options: {:?}", self);

        let id_map: Box<IDMap<u64, u64>> = Box::new(Echo::new());
        // let id_map: Box<IDMap<u64, u64>> = match self.data_dir {
        //     Some(ref dir) => { Box::new(Echo::new()) },
        //     None => { Box::new(Echo::new()) },
        // };

        let map_set: Box<MapSet<Key<u64>, u64>> = match self.data_dir {
            Some(ref dir) => { 
                let mut variant_store_path = dir.clone();
                variant_store_path.push("s_var_mapset");

                Box::new(RocksDB::new(variant_store_path.to_str().unwrap())) 
            },
            None => { Box::new(TempRocksDB::new()) },
        };

        let db: Box<Database<Value=u64>> = Box::new(DB::with_stores(self.bits, self.tolerance, id_map, map_set));

        let mut router = Router::new();

        router.post("/add", handle_add);
        router.post("/query", handle_query);
        router.post("/delete", handle_delete);

        let mut chain = Chain::new(router);
        chain.link_before(State::<DBKey>::one(db));
        Iron::new(chain).http(&*self.bind).unwrap();
    }
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
            let mx = req.get::<State<DBKey>>().unwrap();

            let mut scalar_results = Vec::with_capacity(req_body.scalars.len());
            let mut db = mx.write().unwrap();

            for scalar in req_body.scalars.into_iter() {
                let added = db.insert(scalar.clone());
                let scalar_result = add::ScalarResult {scalar: scalar, added: added};
                scalar_results.push(scalar_result);
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
            let mx = req.get::<State<DBKey>>().unwrap();

            let mut scalar_results = Vec::with_capacity(req_body.scalars.len());
            let db = mx.write().unwrap();

            for scalar in req_body.scalars.into_iter() {
                match db.get(&scalar) {
                    Some(found) => {
                        let scalar_result = query::ScalarResult {scalar: scalar, found: found};
                        scalar_results.push(scalar_result);
                    },
                    None => {
                        let scalar_result = query::ScalarResult {scalar: scalar, found: HashSet::new()};
                        scalar_results.push(scalar_result);
                    },
                }
            }

            Ok(Response::with((status::Ok, json::encode(&query::Response {scalars: scalar_results}).unwrap())))
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
            let mx = req.get::<State<DBKey>>().unwrap();

            let mut scalar_results = Vec::with_capacity(req_body.scalars.len());
            let mut db = mx.write().unwrap();

            for scalar in req_body.scalars.into_iter() {
                let deleted = db.remove(&scalar);
                let scalar_result = delete::ScalarResult {scalar: scalar, deleted: deleted};
                scalar_results.push(scalar_result);
            }

            Ok(Response::with((status::Ok, json::encode(&delete::Response {scalars: scalar_results}).unwrap())))
        },

        Err(err) => {
            Ok(Response::with((status::BadRequest, format!("Unable to parse JSON: {:?}", err))))
        },
    }
}
