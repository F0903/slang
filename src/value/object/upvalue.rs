use std::fmt::{Debug, Display};

use crate::value::Value;

/// A pointer to a variable in an enclosing scope.
//SAFTEY: Since this points to another Value that lives in the same VM stack, the pointer will always be valid.
#[derive(Clone)]
pub struct Upvalue {
    location: *mut Value,
}

impl Upvalue {
    pub fn new(location: *mut Value) -> Self {
        Self { location }
    }

    pub fn set(&mut self, value: Value) {
        unsafe { *self.location = value }
    }

    pub fn get_ref(&self) -> &Value {
        unsafe { self.location.as_ref_unchecked() }
    }
}

impl PartialEq for Upvalue {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.location == *other.location }
    }
}

impl PartialOrd for Upvalue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        unsafe { (*self.location).partial_cmp(&*other.location) }
    }
}

impl Display for Upvalue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(unsafe { self.location.as_mut_unchecked() }, f)
    }
}

impl Debug for Upvalue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("upvalue -> {:?}", self.location))
    }
}
