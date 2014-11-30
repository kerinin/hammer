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
Usage:  hammer [--bind=<address>] [--bits=<value-bitsize>] [--tolerance=<hamming-distance>] [--lru=<max-values>]
        hammer --help

Options:
        -h, --help                          Show this message
        --bind=<address>                    The port to bind to [default: localhost:3000]
        -b, --bits=<value-bitsize>          The bit size of values [default: 64]
        -t, --tolerance=<hamming-distance>  The max hamming distance between query values and returned values [default: 3]
        --lru=<max-values>                  If set, begin dropping oldest values after <max-values> has been written to DB
", flag_bits: uint, flag_tolerance: uint, flag_lru: Option<uint>)

#[cfg(not(test))]
fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    println!("Runnig with args: {}", args);

    let s = http::server::Server {
        bind: args.flag_bind,
        bits: args.flag_bits,
        tolerance: args.flag_tolerance,
        lru: args.flag_lru,
    };

    s.serve()
}
