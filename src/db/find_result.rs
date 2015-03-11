use std::fmt;
use std::hash;
use std::hash::Hasher;

pub enum FindResult<T> {
    ZeroVariant(T),
    OneVariant(T),
}

impl<T: hash::Hash> hash::Hash for FindResult<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            FindResult::ZeroVariant(ref self_value) => self_value.hash(state),
            FindResult::OneVariant(ref self_value) => self_value.hash(state),
        }
    }
}

impl<T: PartialEq> Eq for FindResult<T> {
}

impl<T: PartialEq> PartialEq for FindResult<T> {
    fn eq(&self, other: &FindResult<T>) -> bool {
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

    fn ne(&self, other: &FindResult<T>) -> bool {
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

impl<T: fmt::Display> fmt::Debug for FindResult<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            FindResult::ZeroVariant(ref self_value) => write!(f, "0:{}", self_value),
            FindResult::OneVariant(ref self_value) => write!(f, "1:{}", self_value),
        }
    }
}

