extern crate num;

use std::cmp::Eq;
use std::clone::Clone;

use db::window::Window;

mod db;
mod binary_iter;

pub use self::db::DB;
pub use self::binary_iter::BinaryIter;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key<T> {
    One(Window, T),
    Zero(Window, T),
}

/// Return a set of single-dimensional permutation variants
///
pub trait SubstitutionVariant: Sized {
    type Iter: Iterator<Item = Self>;

    /// Substitution variants
    ///
    /// Returns an array of all possible single-column permutation of `self`.
    /// Alternately, returns the set of values with Hamming distance `1` from 
    /// `self`
    ///
    fn substitution_variants(&self, dimensions: usize) -> <Self as SubstitutionVariant>::Iter;
}

impl<T> SubstitutionVariant for T where
T: Clone,
BinaryIter<T>: Iterator<Item = T>,
{
    type Iter = BinaryIter<T>;

    fn substitution_variants(&self, dimensions: usize) -> BinaryIter<T> {
        BinaryIter::new(self.clone(), dimensions)
    }
}

