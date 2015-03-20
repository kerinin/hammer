//! Arbitrarily-sized matrix of boolean values
//!
//! Intended primarily to provide efficient hamming distance comparsions between
//! vectors of non-boolean elements, as described in the "HBVerify" alogirthm of
//! Zhang's [HmSearch](http://www.cse.unsw.edu.au/~weiw/files/SSDBM13-HmSearch-Final.pdf)
//! paper.
//!
//! Conversion operations are provided by the `AsBitMatrix` trait.  The `transpose`
//! function converts a matrix from row- to column- major order in memory.
//!
//! # Examples
//!
//! Convert & transpose:
//!
//! ```ignore
//! let left =          vec![0, 1, 2, 4, 5];
//! let right =         vec![100, 1, 2, 4, 5];
//! let left_matrix =   left.as_bitmatrix().transpose();
//! let right_matrix =  right.as_bitmatrix().transpose();
//!
//! use db::value::{Hamming};
//!
//! assert_eq!(left_matrix.hamming(&right_matrix), 1);
//! ```

mod as_bit_matrix;
mod bit_matrix;
mod fmt;
mod ops;
mod value;

// // bitmatrix![[1,2,3], [4,5,6]]
// #[macro_export]
// macro_rules! bit_matrix {
//     [ $( [ $( $x:expr ),* ] ),* ] => { BitMatrix::new(vec![ $( vec![ $( $x ),* ] )* ]) };
// }

#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub struct BitMatrix {
    pub data: Vec<Vec<u8>>,
}

pub trait AsBitMatrix {
    /// Converts Self into a BitMatrix
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let vector =            vec![0, 1, 2, 3];
    /// let matrix =            vector.as_bitmatrix();
    /// ```
    fn as_bit_matrix(self) -> BitMatrix;

    /// Converts a BitMatrix into Self
    ///
    /// # Examples
    ///
    /// ```ignore
    // # use bit_matrix::BitMatrix;
    /// let matrix =            BitMatrix::new(vec![vec![0u8, 1u8, 2u8, 3u8]]);
    /// let vector: Vec<u64> =  AsBitMatrix::from_bit_matrix(matrix);
    /// ```
    fn from_bit_matrix(BitMatrix) -> Self;
}

