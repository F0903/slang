use std::ops::{Deref, DerefMut};

use crate::{
    memory::{GC, GcPtr},
    value::{Object, object::AsObjectPtr},
};

pub struct ScopedRootObject {
    ptr: GcPtr<Object>,
}

impl ScopedRootObject {
    pub fn from_ptr(value: impl AsObjectPtr) -> Self {
        let ptr = value.as_object_ptr();
        GC.register_temp_root(ptr);
        Self { ptr }
    }

    /// Takes the inner pointer, unrooting and returning it.
    pub fn take(self) -> GcPtr<Object> {
        self.ptr
    }
}

impl Drop for ScopedRootObject {
    fn drop(&mut self) {
        GC.unregister_temp_root(self.ptr);
    }
}

impl Deref for ScopedRootObject {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        self.ptr.deref()
    }
}

impl DerefMut for ScopedRootObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.deref_mut()
    }
}
