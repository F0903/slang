use std::ops::{Deref, DerefMut};

use crate::{memory::GC, value::object::AsObjectPtr};

pub struct GcScopedRoot<T>
where
    T: AsObjectPtr + Copy,
{
    value: T,
}

impl<T> GcScopedRoot<T>
where
    T: AsObjectPtr + Copy,
{
    #[inline]
    pub fn register(value: T) -> Self {
        GC.register_temp_root(value.as_object_ptr());
        Self { value }
    }

    /// Gets the inner object. The object is still temp rooted for the remainder of the scope.
    #[inline]
    pub fn get_object(&self) -> T {
        self.value
    }
}

impl<T> Drop for GcScopedRoot<T>
where
    T: AsObjectPtr + Copy,
{
    #[inline]
    fn drop(&mut self) {
        GC.unregister_temp_root(self.value.as_object_ptr());
    }
}

impl<T> Deref for GcScopedRoot<T>
where
    T: AsObjectPtr + Copy + Deref,
{
    type Target = <T as Deref>::Target;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

impl<T> DerefMut for GcScopedRoot<T>
where
    T: AsObjectPtr + Copy + DerefMut,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.deref_mut()
    }
}
