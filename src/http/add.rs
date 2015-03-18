use std::clone;
use std::fmt;

#[derive(RustcDecodable)]
pub struct Request {
    pub scalars: Vec<usize>,
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

#[derive(RustcEncodable)]
pub struct Response {
    pub scalars: Vec<ScalarResult>,
}

#[derive(RustcEncodable)]
pub struct ScalarResult {
    pub scalar: usize,
    pub added: bool,
}
