#![feature(collections)]
#![feature(core)]

/*
#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate docopt;
*/

//extern crate iron;
//extern crate bodyparser;
//extern crate router;
//extern crate persistent;

pub mod db;
//pub mod http;
// pub mod evicting_store;


/*
docopt!(Args derive Debug, "
Usage:  hammer [--bind=<address>] [--bits=<value-bitsize>] [--tolerance=<hamming-distance>] [--lru=<max-values>]
        hammer --help

Options:
        -h, --help                          Show this message
        --bind=<address>                    The port to bind to [default: localhost:3000]
        -b, --bits=<value-bitsize>          The bit size of values [default: 64]
        -t, --tolerance=<hamming-distance>  The max hamming distance between query values and returned values [default: 3]
        --lru=<max-values>                  If set, begin dropping oldest values after <max-values> has been written to DB
", flag_bits: u32, flag_tolerance: u32, flag_lru: Option<u32>);
*/

#[cfg(not(test))]
fn main() {
    //let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

    //println!("Runnig with args: {}", args);

    //let s = http::server::Server {
    //    bind: args.flag_bind,
    //    bits: args.flag_bits,
    //    tolerance: args.flag_tolerance,
    //    lru: args.flag_lru,
    //};

    //s.serve()
}
