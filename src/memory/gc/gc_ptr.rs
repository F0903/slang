use std::{
    fmt::{Debug, Display},
    marker::Unsize,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{
    hashing::Hashable,
    memory::{DropDealloc, GC, HeapPtr},
    value::{Object, Value, object::AsObjectPtr},
};

/// A manual version of Box that allocates with the GC.
/// SAFETY: This is a wrapper around a NonNull pointer, cloning will only clone the pointer, and not the underlying data.
pub struct GcPtr<T: ?Sized> {
    inner: HeapPtr<T>,
}

#[allow(dead_code)]
impl<T> GcPtr<T>
where
    T: Sized,
{
    #[inline]
    pub fn alloc(obj: T) -> Self {
        Self {
            inner: HeapPtr::alloc_in(obj, &GC),
        }
    }

    /// This will take ownership of the object and return it.
    /// This makes the underlying value be exposed to the normal drop rules.
    #[inline]
    pub fn take(self) -> T {
        self.inner.take_in(&GC)
    }
}

#[allow(dead_code)]
impl<T> GcPtr<T>
where
    T: ?Sized,
{
    #[inline]
    pub fn take_box(boxed: Box<T>) -> Self {
        Self {
            inner: HeapPtr::take_box(boxed),
        }
    }

    #[inline]
    pub const fn from_raw(ptr: NonNull<T>) -> Self {
        Self {
            inner: HeapPtr::from_raw(ptr),
        }
    }

    #[inline]
    pub(super) fn dealloc(&mut self) {
        self.inner.dealloc();
    }

    #[inline]
    pub fn dealloc_on_drop(self) -> DropDealloc<T> {
        self.inner.dealloc_on_drop()
    }

    #[inline]
    pub const fn get(&self) -> &T {
        self.inner.get()
    }

    #[inline]
    pub const fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    #[inline]
    pub const fn get_raw(&self) -> NonNull<T> {
        self.inner.get_raw()
    }

    #[inline]
    pub fn get_address(&self) -> usize {
        self.inner.get_address()
    }

    /// Coerce this pointer to a trait object pointer (e.g., GcPtr<dyn Trait>).
    #[inline]
    pub fn as_dyn<D: ?Sized>(&self) -> GcPtr<D>
    where
        T: Unsize<D>,
    {
        GcPtr {
            inner: self.inner.as_dyn(),
        }
    }
}

impl<T> Clone for GcPtr<T>
where
    T: ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Copy for GcPtr<T> where T: ?Sized {}

impl<T> Display for GcPtr<T>
where
    T: ?Sized + Display + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("GCPTR {}", self.inner))
    }
}

impl<T> Debug for GcPtr<T>
where
    T: ?Sized + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("GCPTR {:?}  ", self.inner))
    }
}

impl<T> Deref for GcPtr<T>
where
    T: ?Sized,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T> DerefMut for GcPtr<T>
where
    T: ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

impl<T> PartialEq for GcPtr<T>
where
    T: ?Sized,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Hashable for GcPtr<T>
where
    T: ?Sized + Hashable,
{
    #[inline]
    fn get_hash(&self) -> u32 {
        T::get_hash(self.get())
    }
}

impl GcPtr<Object> {
    pub const fn to_value(self) -> Value {
        Value::object(self)
    }
}

impl AsObjectPtr for GcPtr<Object> {
    #[inline]
    fn as_object_ptr(&self) -> GcPtr<Object> {
        self.clone()
    }
}
