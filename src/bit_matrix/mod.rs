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
    fn as_bit_matrix(self) -> BitMatrix;
    fn from_bit_matrix(BitMatrix) -> Self;
}

