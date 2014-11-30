#![feature(phase)]

extern crate iron;
extern crate bodyparser;
extern crate router;
extern crate serialize;
extern crate persistent;
extern crate docopt;

#[phase(plugin)] extern crate docopt_macros;

use docopt::Docopt;

pub mod db;
pub mod http;


docopt!(Args deriving Show, "
Usage:  hammer [--bind=<address>] [--bits=<value-bitsize>] [--tolerance=<hamming-distance>] [--max-values=<db-size>]
        hammer --help

Options:
        -h, --help                          Show this message
        --bind=<address>                    The port to bind to [default: localhost:3000]
        -b, --bits=<value-bitsize>          The bit size of values [default: 64]
        -t, --tolerance=<hamming-distance>  The max hamming distance between query values and returned values [default: 3]
        -m, --max-values=<db-size>          The maximum number of values to store [default: 100000]
")

#[cfg(not(test))]
fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    println!("{}", args);

    http::server::serve()
}
