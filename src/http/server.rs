use serialize::json;
use std::collections::HashSet;

use iron::status;
use iron::prelude::{Set, Plugin, Chain, Request, Response, IronResult, Iron};
use iron::typemap::Assoc;
use iron::response::modifiers::{Status, Body};
use iron::middleware::ChainBuilder;

use router::{Router};
use bodyparser::BodyParser;
use persistent::{State};

use super::add;
use super::query;
use super::delete;
use db::partitioning::Partitioning;

pub struct DB;
impl Assoc<Partitioning<uint>> for DB {}

pub fn serve() {
    let mut router = Router::new();
    let db = Partitioning::new(64, 4);

    router.post("/add", handle_add);
    router.post("/query", handle_query);
    router.post("/delete", handle_delete);

    let mut chain = ChainBuilder::new(router);
    chain.link(State::<DB, Partitioning<uint>>::both(db));
    let server = Iron::new(chain);

    match server.listen("localhost:3000") {
        Ok(..) => println!("Started Iron HTTP Server on localhost:3000"),
        Err(e) => println!("Unable to start HTTP Server: {}", e),
    }
}

fn handle_add(req: &mut Request) -> IronResult<Response> {
    let mutex = req.get::<State<DB, Partitioning<uint>>>().unwrap();

    match req.get::<BodyParser<add::Request>>() {
        Some(parsed) => {
            let mut scalar_results = Vec::with_capacity(parsed.scalars.len());
            let mut db = mutex.write();
            
            for scalar in parsed.scalars.iter() {
                let added = db.insert(*scalar);
                let scalar_result = add::ScalarResult {scalar: *scalar, added: added};
                scalar_results.push(scalar_result);
            }

            Ok(Response::new()
               .set(Status(status::Ok))
               .set(Body(json::encode(&add::Response {scalars: scalar_results})))
            )
        },
        None => {
            Ok(Response::new()
               .set(Status(status::BadRequest))
               .set(Body("Unable to parse JSON"))
            )
        },
    }
}

fn handle_query(req: &mut Request) -> IronResult<Response> {
    let mutex = req.get::<State<DB, Partitioning<uint>>>().unwrap();

    match req.get::<BodyParser<query::Request>>() {
        Some(parsed) => {
            let mut scalar_results = Vec::with_capacity(parsed.scalars.len());
            let db = mutex.read();
            
            for scalar in parsed.scalars.iter() {
                match db.get(*scalar) {
                    Some(found) => {
                        let scalar_result = query::ScalarResult {scalar: *scalar, found: found};
                        scalar_results.push(scalar_result);
                    },
                    None => {
                        let scalar_result = query::ScalarResult {scalar: *scalar, found: HashSet::new()};
                        scalar_results.push(scalar_result);
                    },
                }
            }

            Ok(Response::new()
               .set(Status(status::Ok))
               .set(Body(json::encode(&query::Response {scalars: scalar_results})))
            )
        },
        None => {
            Ok(Response::new()
               .set(Status(status::BadRequest))
               .set(Body("Unable to parse JSON"))
            )
        },
    }
}

fn handle_delete(req: &mut Request) -> IronResult<Response> {
    let mutex = req.get::<State<DB, Partitioning<uint>>>().unwrap();

    match req.get::<BodyParser<delete::Request>>() {
        Some(parsed) => {
            let mut scalar_results = Vec::with_capacity(parsed.scalars.len());
            let mut db = mutex.write();
            
            for scalar in parsed.scalars.iter() {
                let deleted = db.remove(*scalar);
                let scalar_result = delete::ScalarResult {scalar: *scalar, deleted: deleted};
                scalar_results.push(scalar_result);
            }

            Ok(Response::new()
               .set(Status(status::Ok))
               .set(Body(json::encode(&delete::Response {scalars: scalar_results})))
            )
        },
        None => {
            Ok(Response::new()
               .set(Status(status::BadRequest))
               .set(Body("Unable to parse JSON"))
            )
        },
    }
}
