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
    #[inline]
    pub fn new(location: NonNull<Value>) -> Self {
        Self {
            location,
            closed: Value::none(),
            next: None,
        }
    }

    #[inline]
    pub fn new_with_next(location: NonNull<Value>, next: ObjectRef<Upvalue>) -> Self {
        Self {
            location,
            closed: Value::none(),
            next: Some(next),
        }
    }

    #[inline]
    pub(crate) const fn get_location_raw(&self) -> NonNull<Value> {
        self.location
    }

    #[inline]
    pub(crate) fn get_next(&self) -> Option<ObjectRef<Upvalue>> {
        self.next.clone()
    }

    #[inline]
    pub const fn set(&mut self, value: Value) {
        // SAFETY: self.location is guaranteed to be non-null and valid, as it points to a value in the VM stack.
        unsafe {
            *self.location.as_ptr() = value;
        }
    }

    #[inline]
    pub const fn set_next(&mut self, next: Option<ObjectRef<Upvalue>>) {
        self.next = next;
    }

    #[inline]
    pub fn close(&mut self) {
        // We "close" the upvalue by moving the value from the stack to the Upvalue here, which is heap allocated.
        // SAFETY: self.location is guaranteed to be non-null and valid, as it points to a value in the VM stack.
        unsafe {
            self.closed = *self.location.as_ptr();
            // SAFETY: closed is guaranteed to be valid.
            self.location = NonNull::new_unchecked(&raw mut self.closed);
        }
    }

    #[inline]
    pub const fn get_value(&self) -> Value {
        // SAFETY: self.location is guaranteed to be non-null and valid, as it points to a value in the VM stack.
        unsafe { self.location.read() }
    }
}

impl PartialEq for Upvalue {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location
    }
}

impl PartialOrd for Upvalue {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // SAFETY: self.location and other.location are guaranteed to be non-null and valid, as they point to values in the VM stack.
        unsafe { self.location.as_ref().partial_cmp(other.location.as_ref()) }
    }
}

impl Display for Upvalue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SAFETY: self.location is guaranteed to be non-null and valid, as it points to a value in the VM stack.
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
