extern crate byteorder;

use bit_matrix::{BitMatrix, AsBitMatrix};

use self::byteorder::{ByteOrder, LittleEndian};

impl AsBitMatrix for u64 {
    fn as_bit_matrix(self) -> BitMatrix {
        let mut buf = vec![0; 8];
        <LittleEndian as ByteOrder>::write_u64(&mut buf, self);

        BitMatrix::new(vec![buf])
    }

    fn from_bit_matrix(bm: BitMatrix) -> Self {
        // NOTE: Should probably sanity-check the bitmatrix dimensions
        <LittleEndian as ByteOrder>::read_u64(bm.data[0].as_slice())
    }
}

#[cfg(test)]
mod test {
    extern crate quickcheck;
    use self::quickcheck::quickcheck;

    use bit_matrix::AsBitMatrix;

    #[test]
    fn as_bitmatrix_identity_u64() {
        fn prop(a: u64, b: u64) -> quickcheck::TestResult {
            let a_as_bm = a.as_bit_matrix();
            let b_as_bm = b.as_bit_matrix();

            if a == b {
                quickcheck::TestResult::from_bool(a_as_bm == b_as_bm)
            } else {
                quickcheck::TestResult::from_bool(a_as_bm != b_as_bm)
            }
        }
        quickcheck(prop as fn(u64, u64) -> quickcheck::TestResult);
    }

    #[test]
    fn as_bitmatrix_translation_u64() {
        fn prop(a: u64) -> quickcheck::TestResult {
            let as_bm = a.as_bit_matrix();
            let back: u64 = AsBitMatrix::from_bit_matrix(as_bm);

            quickcheck::TestResult::from_bool(a == back)
        }
        quickcheck(prop as fn(u64) -> quickcheck::TestResult);
    }
}
