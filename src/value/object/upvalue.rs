use std::fmt::{Debug, Display};

use crate::{
    dbg_println,
    value::{Value, object::ObjectRef},
};

/// A pointer to a variable in an enclosing scope.
//SAFTEY: Since this points to another Value that lives in the same VM stack, the pointer will always be valid.
#[derive(Clone)]
pub struct Upvalue {
    location: *mut Value,
    closed: Value,
    next: Option<ObjectRef<Upvalue>>,
}

impl Upvalue {
    pub fn new(location: *mut Value) -> Self {
        Self {
            location,
            closed: Value::none(),
            next: None,
        }
    }

    pub fn new_with_next(location: *mut Value, next: ObjectRef<Upvalue>) -> Self {
        Self {
            location,
            closed: Value::none(),
            next: Some(next),
        }
    }

    pub(crate) const fn get_location_raw(&self) -> *const Value {
        self.location
    }

    pub(crate) fn get_next(&self) -> Option<ObjectRef<Upvalue>> {
        self.next.clone()
    }

    pub const fn set(&mut self, value: Value) {
        unsafe { *self.location = value }
    }

    pub const fn set_next(&mut self, next: Option<ObjectRef<Upvalue>>) {
        self.next = next;
    }

    pub fn close(&mut self) {
        // We "close" the upvalue by moving the value from the stack to the Upvalue here, which is heap allocated.
        unsafe {
            self.closed = *self.location;
            self.location = &raw mut self.closed;
        }
    }

    pub const fn get_value(&self) -> Value {
        unsafe { self.location.read() }
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

impl Drop for Upvalue {
    fn drop(&mut self) {
        dbg_println!("DEBUG UPVALUE DROP")
    }
}
