use std::fmt::Display;

use crate::memory::Dealloc;

use super::RawString;

#[derive(Debug)]
pub enum Object {
    String(RawString),
}

impl Dealloc for Object {
    fn dealloc(&mut self) {
        match self {
            Self::String(x) => {
                x.dealloc();
            }
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
