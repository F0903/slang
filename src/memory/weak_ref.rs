use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::memory::HeapPtr;

#[derive(Debug, Clone, Copy)]
pub struct WeakRef<T: Debug> {
    inner: HeapPtr<T>,
}

impl<T: Debug> WeakRef<T> {
    pub(super) fn new(inner: HeapPtr<T>) -> Self {
        Self { inner }
    }
}

impl<T: Debug> Deref for WeakRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<T: Debug> DerefMut for WeakRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}
