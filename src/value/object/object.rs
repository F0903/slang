use std::fmt::{Debug, Display};

use super::StringObject;

pub enum Object {
    String(StringObject),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Object::String(x) => match other {
                Object::String(y) => x == y,
            },
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(x) => f.write_fmt(format_args!("String = {}", x.get_str())),
        }
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(x) => f.write_fmt(format_args!("String = {}", x.get_str())),
        }
    }
}
