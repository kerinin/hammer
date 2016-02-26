extern crate num;

use std::clone::*;
use std::hash::*;

mod db;
mod deletion_iter;
mod xor_iter;

pub use self::db::DB;
pub use self::deletion_iter::DeletionIter;
pub use self::xor_iter::XORIter;

use db::window::Window;

pub type Du8 = (u8, u8);
pub type Du32 = (u32, u8);
pub type Du64 = (u64, u8);
pub type Du64x2 = ([u64; 2], u8);
pub type Du64x4 = ([u64; 4], u8);
pub type Dvec = u64;

pub type Key<T> = (Window, T);

pub trait DeletionVariant<T>: Sized {
    type Iter: Iterator<Item=T>;

    /// Returns an array of all possible deletion variants of `self`
    ///
    /// A "deletion variant" as defined in
    /// [Zhang](http://www.cse.unsw.edu.au/~weiw/files/SSDBM13-HmSearch-Final.pdf)
    /// is a value obtained by substituting a "deletion marker" for a single 
    /// dimension of a value.
    ///
    fn deletion_variants(&self, dimensions: usize) -> <Self as DeletionVariant<T>>::Iter;
}

impl<T, V> DeletionVariant<V> for T where 
T: Clone,
DeletionIter<T>: Iterator<Item = V>
{
    type Iter = DeletionIter<T>;

    fn deletion_variants(&self, dimensions: usize) -> DeletionIter<T> {
        DeletionIter::new(self.clone(), dimensions)
    }
}

impl<T, V> DeletionVariant<V> for Vec<T> where
T: Hash,
Vec<T>: Clone,
XORIter<Vec<T>>: Iterator<Item = V>,
{
    type Iter = XORIter<Vec<T>>;

    fn deletion_variants(&self, dimensions: usize) -> XORIter<Vec<T>> {
        XORIter::new(self.clone(), dimensions)
    }
}

