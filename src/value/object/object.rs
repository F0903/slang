use std::fmt::{Debug, Display};

use super::{InternedString, function::Function};
use crate::memory::Dealloc;

pub enum Object {
    String(InternedString),
    Function(Function),
}

impl Dealloc for Object {
    fn dealloc(&mut self) {
        match self {
            Self::String(_string) => (), // Since all strings are interned and pointing to shared memory, we don't want to dealloc here
            Self::Function(function) => {
                function.dealloc();
            }
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Object::String(x) => match other {
                Object::String(y) => x == y,
                _ => false,
            },
            Object::Function(x) => match other {
                Object::Function(y) => x == y,
                _ => false,
            },
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(x) => f.write_fmt(format_args!("String = {}", x.as_str())),
            Object::Function(x) => f.write_fmt(format_args!("Function = {}", x.get_name())),
        }
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(x) => f.write_fmt(format_args!("String = {}", x.as_str())),
            Object::Function(x) => f.write_fmt(format_args!("Function = {}", x.get_name())),
        }
    }
}
