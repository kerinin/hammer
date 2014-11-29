extern crate iron;
extern crate bodyparser;
extern crate router;
extern crate serialize;

use std::io::net::ip::Ipv4Addr;

use self::iron::prelude::{Set, Plugin, ErrorRefExt, Chain, Request, Response, IronResult, IronError, Iron};
use self::iron::status;

//use self::iron::{Iron, Request, Response, IronResult, Plugin, status};
//
//use self::iron::{Iron, Request, Response, IronResult};
use self::iron::response::modifiers::{Status, Body};

use self::router::{Router};
use self::bodyparser::BodyParser;

pub fn serve() {
    //let logger = Logger::new(None);
    let mut router = Router::new();

    router.post("/add", handleAdd);
    //router.post("/query", handleQuery);
    //router.post("/delete", handleDelete);

    println!("Starting Iron HTTP Server on 127.0.0.1:3000");

    let mut server = Iron::new(router);
    //server.chain.link(logger);
    //server.chain.link(router);
    server.listen("localhost:3000");
}

fn handleAdd(req: &mut Request) -> IronResult<Response> {
    //match req.get::<BodyParser<add::Request>>() {
    //    Some(parsed) => println!("Parsed Json:\n{}", parsed),
    //    None => println!("could not parse"),
    //}

    Ok(Response::new().set(Status(status::Ok)).set(Body("Hello World!")))
}

//fn handleQuery(req: &mut Request) -> IronResult<Response> {
//}
//
//fn handleDelete(req: &mut Request) -> IronResult<Response> {
//}
