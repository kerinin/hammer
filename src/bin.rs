/*
extern crate docopt;
extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate persistent;
extern crate rustc_serialize;

mod db;
mod http;

use docopt::Docopt;

const USAGE: &'static str = "
Hammer

Usage:
  hammer
  hammer (-h | --help)

Options:
  --bind=<host:port>    Host & port to bind to [default: localhost:3000]
  --tolerance=<n>       The match tolerance in bits [default: 7]    
  --bits=<n>            The number of bits to index [default: 64]
  -h --help             Show this screen.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_bind: String,
    flag_bits: usize,
    flag_tolerance: usize,
}

pub fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let s = http::server::Server{
        bind: args.flag_bind,
        bits: args.flag_bits,
        tolerance: args.flag_tolerance,
    };

    s.serve()
}
*/
fn main() {}
