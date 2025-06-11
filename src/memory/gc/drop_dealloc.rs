use std::ops::{Deref, DerefMut};

use crate::{dbg_println, memory::HeapPtr};

/// A wrapper around a GcPtr that automatically deallocates the memory when dropped.
pub struct DropDealloc<T: ?Sized> {
    inner: HeapPtr<T>,
}

impl<T: ?Sized> DropDealloc<T> {
    pub const fn new(value: HeapPtr<T>) -> Self {
        Self { inner: value }
    }
}

impl<T: ?Sized> Deref for DropDealloc<T> {
    type Target = HeapPtr<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> DerefMut for DropDealloc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: ?Sized> Drop for DropDealloc<T> {
    fn drop(&mut self) {
        dbg_println!("DROPDEALLOC DROP");
        self.inner.dealloc();
    }
}
