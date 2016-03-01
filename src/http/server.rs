use std::clone::Clone;
use std::collections::HashMap;

use iron::prelude::*;
use router::Router;
use persistent::State;

use hammer::db::BinaryDB;

use http::{Config, ConfigKey, B32, B64, B128, B256, V32, V64, V128, V256};
use http::binary_handler;
// use http::vector_handler;

pub fn serve(config: Config) {
    println!("Serving with config: {:?}", config);

    let mut router = Router::new();
    router.post("/add/b/:bits/:tolerance/:namespace", binary_handler::add);
    router.post("/query/b/:bits/:tolerance/:namespace", binary_handler::query);
    router.post("/delete/b/:bits/:tolerance/:namespace", binary_handler::delete);

    /*
       router.post("/add/v/:bits/:dimensions/:tolerance/:namespace", vector_handler::add);
       router.post("/query/v/:bits/:dimensions/:tolerance/:namespace", vector_handler::query);
       router.post("/delete/v/:bits/:dimensions/:tolerance/:namespace", vector_handler::delete);
       */

    let mut chain = Chain::new(router);
    chain.link_before(State::<ConfigKey>::one(config.clone()));

    chain.link_before(State::<B256>::one(HashMap::new()));
    chain.link_before(State::<B128>::one(HashMap::new()));
    chain.link_before(State::<B64>::one(HashMap::new()));
    chain.link_before(State::<B32>::one(HashMap::new()));

    chain.link_before(State::<V256>::one(HashMap::new()));
    chain.link_before(State::<V128>::one(HashMap::new()));
    chain.link_before(State::<V64>::one(HashMap::new()));
    chain.link_before(State::<V32>::one(HashMap::new()));

    Iron::new(chain).http(&*config.bind).unwrap();
}
