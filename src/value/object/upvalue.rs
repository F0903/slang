use std::{
    fmt::{Debug, Display},
    ptr::NonNull,
};

use crate::{
    dbg_println,
    value::{Value, object::ObjectRef},
};

/// A pointer to a variable in an enclosing scope.
//SAFTEY: Since this points to another Value that lives in the same VM stack, the pointer will always be valid.
#[derive(Clone)]
pub struct Upvalue {
    location: NonNull<Value>,
    closed: Value,
    next: Option<ObjectRef<Upvalue>>,
}

impl Upvalue {
    pub fn new(location: NonNull<Value>) -> Self {
        Self {
            location,
            closed: Value::none(),
            next: None,
        }
    }

    pub fn new_with_next(location: NonNull<Value>, next: ObjectRef<Upvalue>) -> Self {
        Self {
            location,
            closed: Value::none(),
            next: Some(next),
        }
    }

    pub(crate) const fn get_location_raw(&self) -> NonNull<Value> {
        self.location
    }

    pub(crate) fn get_next(&self) -> Option<ObjectRef<Upvalue>> {
        self.next.clone()
    }

    pub const fn set(&mut self, value: Value) {
        unsafe {
            *self.location.as_ptr() = value;
        }
    }

    pub const fn set_next(&mut self, next: Option<ObjectRef<Upvalue>>) {
        self.next = next;
    }

    pub fn close(&mut self) {
        // We "close" the upvalue by moving the value from the stack to the Upvalue here, which is heap allocated.
        unsafe {
            self.closed = *self.location.as_ptr();
            // SAFETY: closed is guaranteed to be valid.
            self.location = NonNull::new_unchecked(&raw mut self.closed);
        }
    }

    pub const fn get_value(&self) -> Value {
        unsafe { self.location.read() }
    }
}

impl PartialEq for Upvalue {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location
    }
}

impl PartialOrd for Upvalue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        unsafe { self.location.as_ref().partial_cmp(other.location.as_ref()) }
    }
}

impl Display for Upvalue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(unsafe { self.location.as_ref() }, f)
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
