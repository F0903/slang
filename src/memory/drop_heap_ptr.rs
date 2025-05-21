use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::dbg_println;

use super::{Dealloc, HeapPtr};

pub struct DropHeapPtr<T: Debug> {
    ptr: HeapPtr<T>,
}

impl<T> DropHeapPtr<T>
where
    T: Debug,
{
    pub const fn new(ptr: HeapPtr<T>) -> Self {
        Self { ptr }
    }
}

impl<T> Deref for DropHeapPtr<T>
where
    T: Debug,
{
    type Target = <HeapPtr<T> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.ptr.deref()
    }
}

impl<T> DerefMut for DropHeapPtr<T>
where
    T: Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.deref_mut()
    }
}

impl<T> Drop for DropHeapPtr<T>
where
    T: Debug,
{
    fn drop(&mut self) {
        dbg_println!("DROPPING HEAPPTR: {:?}", self.ptr);
        self.ptr.dealloc();
    }
}
