use std::num::Int;

use db::value::Value;

pub struct DeletedValue<V> {
    value: V,
    deleted_dimension: usize,
}

impl<V: Value> DeletedValue<V> {
    fn window(&self, start_dimension: usize, dimensions: usize) -> Self {
        DeletedValue {
            value: self.value.window(start_dimension, dimensions),
            deleted_dimension: self.deleted_dimension,
        }
    }

    fn permutations(&self, dimensions: usize) -> Vec<DeletedValue<V>> {
        return range(0, dimensions)
            .map(|i| {
                DeletedValue { value: self.value.clone(), deleted_dimension: i }
            })
            .collect::<Vec<DeletedValue<V>>>();
    }
}

impl DeletedValue<u8> {
    fn hamming(&self, other: &DeletedValue<u8>) -> usize {
        let self_mask = 1u8 << self.deleted_dimension;
        let other_mask = 1u8 << other.deleted_dimension;

        let self_without_deleted = self.value | self_mask | other_mask;
        let other_without_deleted = other.value | self_mask | other_mask;

        let hamming_without_deleted = (self_without_deleted ^ other_without_deleted).count_ones() as usize;

        if self.deleted_dimension == other.deleted_dimension {
            hamming_without_deleted
        } else {
            2 + hamming_without_deleted
        }
    }
}

#[cfg(test)]
mod test {
    use db::deleted_value::DeletedValue;

    #[test]
    fn test_hamming_zero() {
        let a = DeletedValue {
            value: 0b10000001u8,
            deleted_dimension: 0,
        };
        let b = DeletedValue {
            value: 0b10000001u8,
            deleted_dimension: 0,
        };

        assert_eq!(a.hamming(&b), 0);
    }

    #[test]
    fn test_hamming_one() {
        let a = DeletedValue {
            value: 0b10000001u8,
            deleted_dimension: 0,
        };
        let b = DeletedValue {
            value: 0b00000001u8,
            deleted_dimension: 0,
        };

        assert_eq!(a.hamming(&b), 1);
    }

    #[test]
    fn test_hamming_shared_deletion() {
        let a = DeletedValue {
            value: 0b10000001u8,
            deleted_dimension: 0,
        };
        let b = DeletedValue {
            value: 0b10000000u8,
            deleted_dimension: 0,
        };

        assert_eq!(a.hamming(&b), 0);
    }

    #[test]
    fn test_hamming_different_deletion() {
        let a = DeletedValue {
            value: 0b10000001u8,
            deleted_dimension: 0,
        };
        let b = DeletedValue {
            value: 0b10000000u8,
            deleted_dimension: 1,
        };

        assert_eq!(a.hamming(&b), 2);
    }
}
