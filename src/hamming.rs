extern crate num;

pub trait Hamming<T> {
    fn hamming(&self, rhs: &T) -> uint;
}

impl Hamming<Vec<u8>> for Vec<u8> {
    fn hamming(&self, other: &Vec<u8>) -> uint {
        return 0;
    }
}
