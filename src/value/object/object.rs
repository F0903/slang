use std::fmt::{Debug, Display};

use super::{InternedString, function::Function, native_function::NativeFunction};
use crate::{
    memory::Dealloc,
    value::object::{self, Closure},
};

#[derive(Clone)]
pub enum Object {
    String(InternedString),
    Function(Function),
    NativeFunction(NativeFunction),
    Closure(Closure),
    Upvalue(object::Upvalue),
}

impl Dealloc for Object {
    fn dealloc(&mut self) {
        match self {
            Self::String(_) => (), // Since all strings are interned and pointing to shared memory, we don't want to dealloc here.
            Self::Function(function) => {
                function.dealloc();
            }
            Self::NativeFunction(_) => (),
            Self::Closure(_) => (),
            Self::Upvalue(_) => (),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::String(s) => f.write_str(s.as_str()),
            Object::Function(func) => Display::fmt(func, f),
            Object::NativeFunction(func) => Display::fmt(func, f),
            Object::Closure(clo) => Display::fmt(clo, f),
            Object::Upvalue(up) => Display::fmt(up, f),
        }
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}
