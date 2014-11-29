pub mod db;
pub mod http;

#[cfg(not(test))]
fn main() {
    http::server::serve()
}
