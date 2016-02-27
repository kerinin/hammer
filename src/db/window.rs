use std;
use std::mem::size_of;

#[derive(Clone, Debug, PartialEq, Eq, Hash, RustcDecodable, RustcEncodable)]
pub struct Window {
    pub start_dimension: usize,
    pub dimensions: usize,
}

pub trait Windowable<T> {
    /// Subsample on a set of dimensions
    ///
    /// `start_dimension` the index of the 1st dimension to include in the slice, 
    ///      0-indexed from least significant
    /// `dimensions` the total number of dimensions to include
    ///
    fn window(&self, start_dimension: usize, dimensions: usize) -> T;
}

macro_rules! window_uint_to_uint {
    ($elem:ident, $out:ident) => {
        impl Windowable<$out> for $elem {
            fn window(&self, start_dimension: usize, dimensions: usize) -> $out {
                // source start in range
                assert!(start_dimension < (8 * size_of::<$elem>()));                  
                // source end in range
                assert!(start_dimension + dimensions <= (8 * size_of::<$elem>()));    
                // dimensions in range
                assert!(dimensions <= (8 * size_of::<$out>()));                      

                //  2/5        11111111
                //              ^<--^
                //  << 1       11111110
                //  >> 1+2     00011111
                let trim_high = (8 * size_of::<$elem>()) - (start_dimension + dimensions);

                ((self << trim_high) >> (trim_high + start_dimension)) as $out
            }
        }
    }
}
window_uint_to_uint!(u64, u64);
window_uint_to_uint!(u64, u32);
window_uint_to_uint!(u64, u16);
window_uint_to_uint!(u64, u8);
window_uint_to_uint!(u32, u32);
window_uint_to_uint!(u32, u16);
window_uint_to_uint!(u32, u8);
window_uint_to_uint!(u16, u16);
window_uint_to_uint!(u16, u8);
window_uint_to_uint!(u8, u8);

impl Windowable<Vec<u8>> for Vec<u8> {
    fn window(&self, start_dimension: usize, dimensions: usize) -> Vec<u8> {
        self[start_dimension..(start_dimension + dimensions)].to_vec()
    }
}

pub const ONES_U16: u16 = std::u16::MAX;
pub const EVEN_U16: u16 = 0b1010101010101010u16;
pub const ODD_U16: u16 = 0b0101010101010101u16;
pub const ONES_U8: u8 = std::u8::MAX;
pub const EVEN_U8: u8 = 0b10101010u8;
pub const ODD_U8: u8 = 0b01010101u8;

// Implements Windowable<$out> for [$elem; $elems]
//
// Usage notes:
// $elem: the type of the array element, generally u64
// $elems: the number of elements in the array
// $out: the output type. NOTE: size_of<$out>() should be <= size_of<$elem>()
macro_rules! window_fixed_to_uint {
    ([$elem:ident; $elems:expr], $out:ident) => {

        impl Windowable<$out> for [$elem; $elems] {
            fn window(&self, start_dimension: usize, dimensions: usize) -> $out {
                // source start in range
                assert!(start_dimension < (8 * $elems * size_of::<$elem>()));                  
                // source end in range
                assert!(start_dimension + dimensions <= (8 * $elems * size_of::<$elem>()));    
                // dimensions in range
                assert!(dimensions <= (8 * size_of::<$out>()));                      

                // Contruct the output mask
                let mut out = if dimensions == (8 * size_of::<$out>()) {
                    std::$out::MAX
                } else {
                    std::$out::MAX ^ (std::$out::MAX << dimensions)
                };

                let offset = $elems - 1 - (start_dimension / (8 * size_of::<$elem>()));
                let shift = start_dimension % (8 * size_of::<$elem>());

                // AND the shifted bits into the mask
                if shift == 0 {
                    out &= self[offset] as $out
                } else if offset == 0 {
                    out &= (self[offset] >> shift) as $out
                } else {
                    out &= ((self[offset] >> shift) | (self[offset-1] << ((8 * size_of::<$elem>())-shift))) as $out
                }

                return out
            }
        }
    }
}
window_fixed_to_uint!([u64; 4], u64);
window_fixed_to_uint!([u64; 4], u32);
window_fixed_to_uint!([u64; 4], u16);
window_fixed_to_uint!([u64; 4], u8);
window_fixed_to_uint!([u64; 3], u64);
window_fixed_to_uint!([u64; 3], u32);
window_fixed_to_uint!([u64; 3], u16);
window_fixed_to_uint!([u64; 3], u8);
window_fixed_to_uint!([u64; 2], u64);
window_fixed_to_uint!([u64; 2], u32);
window_fixed_to_uint!([u64; 2], u16);
window_fixed_to_uint!([u64; 2], u8);

// This is mostly implemented for testing
window_fixed_to_uint!([u16; 4], u8);

/*
#[cfg(test)]
mod bench {
extern crate test;
extern crate rand;

use self::test::Bencher;
use self::rand::*;

use db::window::*;

#[bench]
fn u64x4_to_u64(b: &mut Bencher) {
let mut rng = thread_rng();
let mut v = [0u64; 4];

b.iter(|| -> u64 {
// RNG overhead is around 240 ns/iter (+/- 24)
v = [rng.gen(), rng.gen(), rng.gen(), rng.gen()];
let start = rng.gen::<usize>() % 192;
let dims = 1 + rng.gen::<usize>() % 64;
v.window(start, dims)
})
}
}
*/

#[cfg(test)] 
mod test {
    extern crate rand;
    extern crate quickcheck;

    use self::quickcheck::quickcheck;

    use db::window::*;

    #[test]
    fn quick_u16_u8() {
        fn prop(x: usize, y: usize) -> quickcheck::TestResult {
            let start_dimension = x % 16;
            let dimensions = y % 8;
            if start_dimension + dimensions > 16 || start_dimension == 16 || dimensions == 0 || dimensions > 8 {
                return quickcheck::TestResult::discard()
            }

            let start = EVEN_U16;
            let actual: u8 = start.window(start_dimension, dimensions);
            let fill = if start_dimension % 2 == 0 { EVEN_U8 } else { ODD_U8 };
            let expected = match dimensions / 8 {
                0 => fill & (ONES_U8 ^ (ONES_U8 << dimensions)),
                1 => fill,
                _ => panic!("wtf"),
            };

            /*
               println!(
               "[{:016b},{:016b},{:016b},{:016b}].window({:2},{:2}) -> {:08b} (exp {:08b})", 
               start[0], start[1], start[2], start[3],
               start_dimension, dimensions, actual, expected,
               );
               */
            quickcheck::TestResult::from_bool(actual == expected)
        }
        quickcheck(prop as fn(usize, usize) -> quickcheck::TestResult);
    }

    #[test]
    fn quick_u16x4_u8() {
        fn prop(start_dimension: usize, dimensions: usize) -> quickcheck::TestResult {
            if start_dimension + dimensions > 64 || start_dimension == 64 || dimensions == 0 || dimensions > 8 {
                return quickcheck::TestResult::discard()
            }


            let start = [EVEN_U16, EVEN_U16, EVEN_U16, EVEN_U16];
            let actual: u8 = start.window(start_dimension, dimensions);
            let fill = if start_dimension % 2 == 0 { EVEN_U8 } else { ODD_U8 };
            let expected = match dimensions / 8 {
                0 => fill & (ONES_U8 ^ (ONES_U8 << dimensions)),
                1 => fill,
                _ => panic!("wtf"),
            };

            /*
               println!(
               "[{:016b},{:016b},{:016b},{:016b}].window({:2},{:2}) -> {:08b} (exp {:08b})", 
               start[0], start[1], start[2], start[3],
               start_dimension, dimensions, actual, expected,
               );
               */
            quickcheck::TestResult::from_bool(actual == expected)
        }
        quickcheck(prop as fn(usize, usize) -> quickcheck::TestResult);
    }
}
