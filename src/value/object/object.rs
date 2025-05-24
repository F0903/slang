use super::InternedString;
use crate::memory::Dealloc;
use std::fmt::{Debug, Display};

pub enum Object {
    String(InternedString),
}

impl Dealloc for Object {
    fn dealloc(&mut self) {
        match self {
            Self::String(_string) => (), // Since all strings are interned and pointing to shared memory, we don't want to dealloc here
        }
    }
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
