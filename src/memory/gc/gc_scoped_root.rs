use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::memory::{GC, GcPtr, Markable};

pub struct GcScopedRoot<T>
where
    T: Markable + Debug + 'static,
{
    value: GcPtr<T>,
}

impl<T> GcScopedRoot<T>
where
    T: Markable + Debug + 'static,
{
    pub fn from_ptr(value: GcPtr<T>) -> Self {
        GC.register_temp_root(value.get_raw().as_ptr());
        Self { value }
    }
}

impl<T> Drop for GcScopedRoot<T>
where
    T: Markable + Debug + 'static,
{
    fn drop(&mut self) {
        GC.unregister_temp_root(self.value.get_raw().as_ptr());
    }
}

impl<T> Deref for GcScopedRoot<T>
where
    T: Markable + Debug + 'static,
{
    type Target = <GcPtr<T> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

impl<T> DerefMut for GcScopedRoot<T>
where
    T: Markable + Debug + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.deref_mut()
    }
}
