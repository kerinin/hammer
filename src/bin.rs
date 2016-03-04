/*
   extern crate docopt;
#[macro_use]
extern crate iron;
extern crate bincode;
extern crate router;
extern crate persistent;
extern crate rustc_serialize;
extern crate hammer;

pub mod http;

use std::path::PathBuf;

use docopt::Docopt;

const USAGE: &'static str = "
Hammer

Usage:
hammerhttp [--data-dir=<path>] [--bind=<host:port>]
hammerhttp (-h | --help)

Options:
--data-dir=<path>     If set, data will be persisted to the given path (if 
unset, data will be persisted to a temporary location)
--bind=<host:port>    Host & port to bind to [default: localhost:3000]
-h --help             Show this screen.
";

#[derive(Debug, RustcDecodable)]
struct Args {
flag_data_dir: Option<String>,
flag_bind: String,
}

pub fn main() {
let args: Args = Docopt::new(USAGE)
.and_then(|d| d.decode())
.unwrap_or_else(|e| e.exit());

let config = http::Config{
data_dir: args.flag_data_dir.map(|d| PathBuf::from(d)),
bind: args.flag_bind,
};

http::server::serve(config)
}
*/
fn main() {}
