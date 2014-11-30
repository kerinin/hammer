extern crate iron;
extern crate bodyparser;
extern crate router;
extern crate serialize;
extern crate persistent;

pub mod db;
pub mod http;

#[cfg(not(test))]
fn main() {
    http::server::serve()
}
