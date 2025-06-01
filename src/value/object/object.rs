use std::fmt::{Debug, Display};

use super::{InternedString, function::Function, native_function::NativeFunction};
use crate::memory::Dealloc;

#[derive(Clone)]
pub enum Object {
    String(InternedString),
    Function(Function),
    NativeFunction(NativeFunction),
}

impl Dealloc for Object {
    fn dealloc(&mut self) {
        match self {
            Self::String(_) => (), // Since all strings are interned and pointing to shared memory, we don't want to dealloc here.
            Self::Function(function) => {
                function.dealloc();
            }
            Self::NativeFunction(_) => (),
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
            Object::NativeFunction(x) => match other {
                Object::NativeFunction(y) => x == y,
                _ => false,
            },
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(x) => f.write_fmt(format_args!("String = {}", x.as_str())),
            Object::Function(x) => f.write_fmt(format_args!(
                "Function = {}",
                x.name
                    .as_ref()
                    .map(|x| x.as_str().to_owned())
                    .unwrap_or("unnamed function".to_owned())
            )),
            Object::NativeFunction(x) => f.write_fmt(format_args!("NativeFunction = {:?}", x)),
        }
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(x) => f.write_fmt(format_args!("String = {:?}", x.as_str())),
            Object::Function(x) => f.write_fmt(format_args!("Function = {:?}", x.name)),
            Object::NativeFunction(x) => f.write_fmt(format_args!("NativeFunction = {:?}", x)),
        }
    }
}
