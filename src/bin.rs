extern crate docopt;
extern crate iron;
extern crate router;
extern crate persistent;
extern crate rustc_serialize;
extern crate hammer;

mod http;

use std::path::PathBuf;

use docopt::Docopt;

const USAGE: &'static str = "
Hammer

Usage:
  hammerhttp [--data-dir=<path>] [--bind=<host:port>] [--tolerance=<n>] [--bits=<n>]
  hammerhttp (-h | --help)

Options:
  --data-dir=<path>     If set, data will be persisted to the given path (if 
                        unset, data will be persisted to a temporary location)
  --bind=<host:port>    Host & port to bind to [default: localhost:3000]
  --tolerance=<n>       The match tolerance in bits [default: 7]    
  --bits=<n>            The number of bits to index [default: 64]
  -h --help             Show this screen.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_data_dir: Option<String>,
    flag_bind: String,
    flag_bits: usize,
    flag_tolerance: usize,
}

pub fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let s = http::server::Server{
        data_dir: args.flag_data_dir.map(|d| PathBuf::from(d)),
        bind: args.flag_bind,
        bits: args.flag_bits,
        tolerance: args.flag_tolerance,
    };

    s.serve()
}
