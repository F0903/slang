use std::fmt::Debug;

use crate::memory::{GC, HeapPtr, Markable};

pub struct GcScopedRoot<T: Markable + Debug + 'static> {
    value: HeapPtr<T>,
}

impl<T: Markable + Debug + 'static> GcScopedRoot<T> {
    pub fn from_ptr(value: HeapPtr<T>) -> Self {
        GC.register_temp_root(value.get_raw().as_ptr());
        Self { value }
    }
}

impl<T: Markable + Debug + 'static> Drop for GcScopedRoot<T> {
    fn drop(&mut self) {
        GC.unregister_temp_root(self.value.get_raw().as_ptr());
    }
}
