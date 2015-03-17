use std::fmt;

use bit_matrix::BitMatrix;

impl fmt::Binary for BitMatrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // Probably a better way of doing multiple writes...
        let _ = write!(f, "[");
        for outer in self.data.iter() {
            let _ = write!(f, "[");
            for inner in outer.iter() {
                let _ = write!(f, "{:08b}", inner);
            }
            let _ = write!(f, "]");
        }
        write!(f, "]")
    }
}
