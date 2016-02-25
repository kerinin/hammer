extern crate num;

use std::cmp::Eq;
use std::clone::Clone;

use db::window::Window;

mod db;
mod binary_iter;

pub use self::db::DB;
pub use self::binary_iter::BinaryIter;

#[derive(Clone, Debug, PartialEq, Eq, Hash, RustcDecodable, RustcEncodable)]
pub enum Key<T> {
    One(Window, T),
    Zero(Window, T),
}

/// Return a set of single-dimensional permutation variants
///
pub trait SubstitutionVariant<V>: Sized {
    type Iter: Iterator<Item = V>;

    /// Returns a non-permuted version of `self` with type `<self as Iterator>::Item`
    ///
    /// This method exists to support compact variants
    ///
    fn null_variant(&self) -> V;

    /// Substitution variants
    ///
    /// Returns an array of all possible single-column permutation of `self`.
    /// Alternately, returns the set of values with Hamming distance `1` from 
    /// `self`
    ///
    fn substitution_variants(&self, dimensions: usize) -> <Self as SubstitutionVariant<V>>::Iter;
}

impl<T> SubstitutionVariant<T> for T where
T: Clone,
BinaryIter<T>: Iterator<Item = T>,
{
    type Iter = BinaryIter<T>;

    fn null_variant(&self) -> T {
        self.clone()
    }

    fn substitution_variants(&self, dimensions: usize) -> BinaryIter<T> {
        BinaryIter::new(self.clone(), dimensions)
    }
}

