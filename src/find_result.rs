extern crate num;

use std::fmt;

pub enum FindResult<T> {
    ZeroVariant(T),
    OneVariant(T),
}

impl<T: PartialEq> PartialEq for FindResult<T> {
    fn eq(&self, other: &FindResult<T>) -> bool {
        match *self {
            ZeroVariant(ref self_value) => match *other {
                ZeroVariant(ref other_value) => self_value.eq(other_value),
                OneVariant(_) => false,
            },
            OneVariant(ref self_value) => match *other {
                OneVariant(ref other_value) => self_value.eq(other_value),
                ZeroVariant(_) => false,
            },
        }
    }

    fn ne(&self, other: &FindResult<T>) -> bool {
        match *self {
            ZeroVariant(ref self_value) => match *other {
                ZeroVariant(ref other_value) => self_value.ne(other_value),
                OneVariant(_) => true,
            },
            OneVariant(ref self_value) => match *other {
                OneVariant(ref other_value) => self_value.ne(other_value),
                ZeroVariant(_) => true,
            },
        }
    }
}

impl<T: fmt::Binary + fmt::Show> fmt::Show for FindResult<Vec<T>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::FormatError> {
        match *self {
            ZeroVariant(ref value) => write!(
                f, "(0:{})", 
                value.iter().map(|b| format!("{:08t}", *b)).collect::<Vec<String>>()
                ),
            OneVariant(ref value) => write!(
                f, "(1:{})", 
                value.iter().map(|b| format!("{:08t}", *b)).collect::<Vec<String>>()
                ),
        }
    }
}
