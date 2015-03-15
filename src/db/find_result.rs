use std::hash;
use std::hash::Hasher;

use db::value::Value;

#[derive(Debug)]
pub enum FindResult<V> {
    ZeroVariant(V),
    OneVariant(V),
}

impl<V: Value> hash::Hash for FindResult<V> {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        match *self {
            FindResult::ZeroVariant(ref self_value) => self_value.hash(state),
            FindResult::OneVariant(ref self_value) => self_value.hash(state),
        }
    }
}

impl<V: Value> Eq for FindResult<V> {
}

impl<V: Value> PartialEq for FindResult<V> {
    fn eq(&self, other: &FindResult<V>) -> bool {
        match *self {
            FindResult::ZeroVariant(ref self_value) => match *other {
                FindResult::ZeroVariant(ref other_value) => self_value.eq(other_value),
                FindResult::OneVariant(_) => false,
            },
            FindResult::OneVariant(ref self_value) => match *other {
                FindResult::OneVariant(ref other_value) => self_value.eq(other_value),
                FindResult::ZeroVariant(_) => false,
            },
        }
    }

    fn ne(&self, other: &FindResult<V>) -> bool {
        match *self {
            FindResult::ZeroVariant(ref self_value) => match *other {
                FindResult::ZeroVariant(ref other_value) => self_value.ne(other_value),
                FindResult::OneVariant(_) => true,
            },
            FindResult::OneVariant(ref self_value) => match *other {
                FindResult::OneVariant(ref other_value) => self_value.ne(other_value),
                FindResult::ZeroVariant(_) => true,
            },
        }
    }
}

/*
impl fmt::Debug for FindResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            FindResult::ZeroVariant(ref self_value) => {
                let first = match self_value.get(0) {
                    Some(value) => value.to_string(),
                    None => "-".to_string(),
                };
                let last = match self_value.get(0) {
                    Some(value) => value.to_string(),
                    None => "-".to_string(),
                };
                let length = self_value.len();

                write!(f, "0:[{}-{}:{}]", first, last, length)
            },
            FindResult::OneVariant(ref self_value) => {
                let first = match self_value.get(0) {
                    Some(value) => value.to_string(),
                    None => "-".to_string(),
                };
                let last = match self_value.get(0) {
                    Some(value) => value.to_string(),
                    None => "-".to_string(),
                };
                let length = self_value.len();

                write!(f, "1:[{}-{}:{}]", first, last, length)
            },
        }
    }
}
// impl fmt::Debug for FindResult {
//     fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
//         match *self {
//             FindResult::ZeroVariant(ref self_value) => write!(f, "0:{}", self_value),
//             FindResult::OneVariant(ref self_value) => write!(f, "1:{}", self_value),
//         }
//     }
// }
*/
