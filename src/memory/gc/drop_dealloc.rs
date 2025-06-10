use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::GcPtr;
use crate::dbg_println;

/// A wrapper around a GcPtr that automatically deallocates the memory when dropped.
pub struct DropDealloc<T: Debug> {
    inner: GcPtr<T>,
}

impl<T> DropDealloc<T>
where
    T: Debug,
{
    pub const fn new(value: GcPtr<T>) -> Self {
        Self { inner: value }
    }
}

impl<T> Deref for DropDealloc<T>
where
    T: Debug,
{
    type Target = <GcPtr<T> as Deref>::Target;

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
