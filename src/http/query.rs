use serialize::json;
use std::clone;
use std::fmt;

use std::collections::HashSet;

#[deriving(Decodable)]
pub struct Request {
    pub scalars: Vec<uint>,
}

impl clone::Clone for Request {
    fn clone(&self) -> Request {
        Request {scalars: self.scalars.clone()}
    }

    fn clone_from(&mut self, source: &Request) {
        self.scalars = source.scalars.clone();
    }
}

impl fmt::Show for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "s:{}", self.scalars)
    }
}

#[deriving(Encodable)]
pub struct Response {
    pub scalars: Vec<ScalarResult>,
}

#[deriving(Encodable)]
pub struct ScalarResult {
    pub scalar: uint,
    pub found: HashSet<uint>,
}
