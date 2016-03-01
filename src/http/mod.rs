pub mod server;

mod binary_handler;
mod vector_handler;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use std::io::Read;

use iron::prelude::*;
use iron::{status, typemap};
use rustc_serialize::base64;
use rustc_serialize::json;
use rustc_serialize::Decodable;
use hammer::db::Database;

pub enum HammerHTTPError {
    Base64DecodeError,
    BincodeDecodeError,
    VectorLengthError,
}

struct B32;
impl typemap::Key for B32 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<u32>>>>>; }
struct B64;
impl typemap::Key for B64 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<u64>>>>>; }
struct B128;
impl typemap::Key for B128 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<[u64; 2]>>>>>; }
struct B256;
impl typemap::Key for B256 { type Value = HashMap<(usize, String), Arc<RwLock<Box<Database<[u64; 4]>>>>>; }

struct V32;
impl typemap::Key for V32 { type Value = HashMap<(usize, usize, String), Arc<RwLock<Box<Database<Vec<u32>>>>>>; }
struct V64;
impl typemap::Key for V64 { type Value = HashMap<(usize, usize, String), Arc<RwLock<Box<Database<Vec<u64>>>>>>; }
struct V128;
impl typemap::Key for V128 { type Value = HashMap<(usize, usize, String), Arc<RwLock<Box<Database<Vec<[u64; 2]>>>>>>; }
struct V256;
impl typemap::Key for V256 { type Value = HashMap<(usize, usize, String), Arc<RwLock<Box<Database<Vec<[u64; 4]>>>>>>; }

const BASE64_CONFIG: base64::Config = base64::Config{
    char_set: base64::CharacterSet::Standard,
    newline: base64::Newline::CRLF,
    pad: true,
    line_length: None,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: Option<PathBuf>,
    pub bind: String,
    pub bits: usize,
    pub tolerance: usize,
}

struct ConfigKey;
impl typemap::Key for ConfigKey { type Value = Config; }

fn decode_body<T>(req: &mut Request) -> Result<T, IronError> where
T: Decodable
{
    let mut payload = String::new();
    itry!(req.body.read_to_string(&mut payload));

    match json::decode::<T>(&payload) {
        Ok(req_body) => {
            Ok(req_body)
        },
        Err(err) => {
            Err(IronError::new(err, (status::BadRequest, "Unable to parse JSON")))
        }
    }
}
