use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::HeapPtr;
use crate::{dbg_println, memory::Dealloc};

// A wrapper around a Dealloc type that automatically deallocates the memory when dropped.
pub struct DropDealloc<T: Dealloc + Debug> {
    inner: T,
}

impl<T> DropDealloc<T>
where
    T: Dealloc + Debug,
{
    pub const fn new(value: T) -> Self {
        Self { inner: value }
    }
}

impl<T> Deref for DropDealloc<T>
where
    T: Dealloc + Debug,
{
    type Target = <HeapPtr<T> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for DropDealloc<T>
where
    T: Dealloc + Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Drop for DropDealloc<T>
where
    T: Dealloc + Debug,
{
    fn drop(&mut self) {
        dbg_println!("DROPPING HEAPPTR: {:?}", self.inner);
        self.inner.dealloc();
    }
}

pub trait DeallocOnDrop {
    fn dealloc_on_drop(self) -> DropDealloc<Self>
    where
        Self: Sized + Dealloc + Debug;
}

impl<T: Dealloc> DeallocOnDrop for T {
    fn dealloc_on_drop(self) -> DropDealloc<Self>
    where
        Self: Sized + Dealloc + Debug,
    {
        DropDealloc::new(self)
    }
}
