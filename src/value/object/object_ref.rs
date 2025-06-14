use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{
    memory::GcPtr,
    value::{Object, Value, object::AsObjectPtr},
};

#[derive(Debug)]
pub struct ObjectRef<T> {
    ///SAFETY: These is guaranteed to live until GC'd, at which point there is no references to the object (hence it was GC'd)
    ptr: NonNull<T>,
    parent: NonNull<Object>,
}

#[allow(dead_code)]
impl<T> ObjectRef<T> {
    #[inline]
    pub(super) fn new(ptr: NonNull<T>, parent: NonNull<Object>) -> Self {
        Self { ptr, parent }
    }

    /// Upcast the object reference to a full Object.
    // SAFETY: Since all objects are heap allocated and managed by the GC, the parent pointer is guaranteed to be valid.
    #[inline]
    pub const fn upcast(&self) -> GcPtr<Object> {
        GcPtr::from_raw(self.parent)
    }

    #[inline]
    pub const fn as_ref(&self) -> &T {
        // SAFETY: The pointer is guaranteed to be non-null and valid as long as the object is alive.
        unsafe { self.ptr.as_ref() }
    }

    #[inline]
    pub const fn as_mut(&mut self) -> &mut T {
        // SAFETY: The pointer is guaranteed to be non-null and valid as long as the object is alive.
        unsafe { self.ptr.as_mut() }
    }

    #[inline]
    pub const fn to_value(self) -> Value {
        Value::object(self.upcast())
    }

    #[inline]
    pub fn addr_gt_addr<A>(&self, other: *const A) -> bool {
        (self.ptr.as_ptr() as usize) > (other as usize)
    }

    #[inline]
    pub fn addr_eq_addr<A>(&self, other: *const A) -> bool {
        (self.ptr.as_ptr() as usize) == (other as usize)
    }

    #[inline]
    pub fn addr_lt_addr<A>(&self, other: *const A) -> bool {
        (self.ptr.as_ptr() as usize) < (other as usize)
    }
}

impl<T> Clone for ObjectRef<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            parent: self.parent,
        }
    }
}

impl<T> Copy for ObjectRef<T> {}

impl<T> Deref for ObjectRef<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for ObjectRef<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T: PartialEq> PartialEq for ObjectRef<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        T::eq(self.as_ref(), other.as_ref())
    }
}

impl<T: Display> Display for ObjectRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        T::fmt(self.as_ref(), f)
    }
}

impl<T> AsObjectPtr for ObjectRef<T> {
    #[inline]
    fn as_object_ptr(&self) -> GcPtr<Object> {
        self.upcast()
    }
}
