use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::HeapPtr;
use crate::dbg_println;

/// A wrapper around a HeapPtr that automatically deallocates the memory when dropped.
pub struct DropDealloc<T: Debug> {
    inner: HeapPtr<T>,
}

impl<T> DropDealloc<T>
where
    T: Debug,
{
    pub const fn new(value: HeapPtr<T>) -> Self {
        Self { inner: value }
    }
}

impl<T> Deref for DropDealloc<T>
where
    T: Debug,
{
    type Target = <HeapPtr<T> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for DropDealloc<T>
where
    T: Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Drop for DropDealloc<T>
where
    T: Debug,
{
    fn drop(&mut self) {
        dbg_println!("DROPDEALLOC DROP: {:?}", self.inner);
        self.inner.dealloc();
    }
}
