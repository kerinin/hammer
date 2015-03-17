use bit_matrix::BitMatrix;

impl BitMatrix {
    pub fn new(data: Vec<Vec<u8>>) -> BitMatrix {
        BitMatrix { data: data }
    }

    pub fn rows(&self) -> usize {
        self.data.len()
    }
    pub fn columns(&self) -> usize {
        self.data[0].len() * 8
    }

    // SUPER inefficient, intended as a placeholder
    pub fn transpose(&self) -> Self {
        let source_x = self.data.len();
        let source_y = self.data[0].len();
        let mut out = vec![vec![0; source_x]; source_y];

        for from_x in 0..source_x {
            for from_y in 0..source_y {
                out[from_y][from_x] = self.data[from_x][from_y];
            }
        }

        return BitMatrix {data: out};
    }

    pub fn mask(&self, dimension: usize) -> BitMatrix {
        let byte_offset = self.data[0].len() - (dimension / 8) - 1;
        let bit_offset = dimension % 8;
        let toggle = 1u8 << bit_offset;
        let mut masked = self.clone();

        for i in 0..self.data.len() {
            masked.data[i][byte_offset] = masked.data[i][byte_offset] ^ toggle;
        }

        masked
    }

    pub fn permute(&self, dimension: usize) -> Vec<BitMatrix> {
        let byte_offset = self.data[0].len() - (dimension / 8) - 1;
        let bit_offset = dimension % 8;
        let toggle = 1u8 << bit_offset;

        (0..self.data.len()).map(|i| {
            let mut permuted = self.clone();
            permuted.data[i][byte_offset] = permuted.data[i][byte_offset] ^ toggle;
            permuted
        }).collect::<Vec<BitMatrix>>()
    }
}

#[cfg(test)]
mod test {
}
