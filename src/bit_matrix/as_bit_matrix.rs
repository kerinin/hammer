extern crate byteorder;

use bit_matrix::{BitMatrix, AsBitMatrix};

use self::byteorder::{ByteOrder, LittleEndian};

impl AsBitMatrix for u64 {
    fn as_bit_matrix(self) -> BitMatrix {
        vec![self].as_bit_matrix()
    }

    fn from_bit_matrix(m: BitMatrix) -> Self {
        let v: Vec<u64> = AsBitMatrix::from_bit_matrix(m);
        v[0]
    }
}
impl AsBitMatrix for Vec<Vec<u8>> {
    fn as_bit_matrix(self) -> BitMatrix {
        BitMatrix::new(self)
    }

    fn from_bit_matrix(m: BitMatrix) -> Self {
        m.data
    }
}
impl AsBitMatrix for Vec<u64> {
    fn as_bit_matrix(self) -> BitMatrix {
        let data = self.iter().map(|i| {
            let mut buf = vec![0; 8];
            <LittleEndian as ByteOrder>::write_u64(&mut buf, *i);
            buf
        }).collect::<Vec<Vec<u8>>>();

        BitMatrix::new(data)
    }

    fn from_bit_matrix(m: BitMatrix) -> Self {
        m.data.iter().map(|i| {
            <LittleEndian as ByteOrder>::read_u64(i.as_slice())
        }).collect::<Vec<u64>>()
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
