use std::collections::HashSet;

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
pub struct Request {
    pub scalars: Vec<u64>,
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
pub struct Response {
    pub scalars: Vec<ScalarResult>,
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
pub struct ScalarResult {
    pub scalar: u64,
    pub found: HashSet<u64>,
}
