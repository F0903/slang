use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{memory::HeapPtr, value::Object};

#[derive(Debug)]
pub struct ObjectRef<T> {
    ///SAFETY: These is guaranteed to live until GC'd, at which point there is no references to the object (hence it was GC'd)
    ptr: *const T,
    parent: NonNull<Object>,
}

impl<T> ObjectRef<T> {
    pub(super) fn new(ptr: *const T, parent: NonNull<Object>) -> Self {
        Self { ptr, parent }
    }

    /// Upcast the object reference to a full Object.
    // SAFETY: Since all objects are heap allocated and managed by the GC, the parent pointer is guaranteed to be valid.
    pub const fn upcast(&self) -> HeapPtr<Object> {
        HeapPtr::from_raw(self.parent)
    }

    pub const fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref_unchecked() }
    }

    pub const fn as_mut(&mut self) -> &mut T {
        // Casting from const to mut here is obviously extra unsafe, but we are riding wild out here.
        unsafe { (self.ptr as *mut T).as_mut_unchecked() }
    }

    pub fn addr_gt_addr<A>(&self, other: *const A) -> bool {
        (self.ptr as usize) > (other as usize)
    }

    pub fn addr_eq_addr<A>(&self, other: *const A) -> bool {
        (self.ptr as usize) == (other as usize)
    }

    pub fn addr_lt_addr<A>(&self, other: *const A) -> bool {
        (self.ptr as usize) < (other as usize)
    }
}

impl<T> Clone for ObjectRef<T> {
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

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for ObjectRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T: PartialEq> PartialEq for ObjectRef<T> {
    fn eq(&self, other: &Self) -> bool {
        T::eq(self.as_ref(), other.as_ref())
    }
}

impl<T: PartialOrd> PartialOrd for ObjectRef<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        T::partial_cmp(self.as_ref(), other.as_ref())
    }
}

impl<T: Display> Display for ObjectRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        T::fmt(self.as_ref(), f)
    }
}
