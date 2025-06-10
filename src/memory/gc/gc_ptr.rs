use std::{
    fmt::{Debug, Display},
    marker::Unsize,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{
    hashing::Hashable,
    memory::{DropDealloc, GC},
    value::{Object, object::AsObjectPtr},
};

/// A manual version of Box that allocates with the GC.
/// SAFETY: This is a wrapper around a NonNull pointer, cloning will only clone the pointer, and not the underlying data.
pub struct GcPtr<T: ?Sized> {
    mem: NonNull<T>,
    #[cfg(debug_assertions)]
    dealloced: bool,
}

impl<T> GcPtr<T>
where
    T: Sized,
{
    pub fn alloc(obj: T) -> Self {
        // Using Box::leak is more efficient than manually allocating due to some internal Rust optimizations.
        Self {
            // SAFETY: This is guaranteed to be non-null, as we are literally creating the Box right here.
            mem: unsafe { NonNull::new_unchecked(Box::leak(Box::new_in(obj, &GC))) },
            #[cfg(debug_assertions)]
            dealloced: false,
        }
    }

    /// This will take ownership of the object and return it.
    /// This makes the underlying value be exposed to the normal drop rules.
    pub fn take(self) -> T {
        let val = unsafe { *Box::from_raw_in(self.mem.as_ptr(), &GC) };
        val
    }
}

impl<T> GcPtr<T>
where
    T: ?Sized,
{
    pub fn dealloc(&mut self) {
        #[cfg(debug_assertions)]
        {
            assert!(!self.dealloced, "Double free detected!");
            self.dealloced = true;
        }
        unsafe {
            if std::mem::needs_drop::<T>() {
                drop(Box::from_raw(self.mem.as_ptr()));
            }
        }
    }

    pub fn dealloc_on_drop(self) -> DropDealloc<T> {
        DropDealloc::new(self)
    }

    pub fn take_box(boxed: Box<T>) -> Self {
        Self {
            mem: unsafe { NonNull::new_unchecked(Box::leak(boxed)) },
            dealloced: false,
        }
    }

    pub const fn from_raw(ptr: NonNull<T>) -> Self {
        Self {
            mem: ptr,
            #[cfg(debug_assertions)]
            dealloced: false,
        }
    }

    pub const fn get(&self) -> &T {
        unsafe { self.mem.as_ref() }
    }

    pub const fn get_mut(&mut self) -> &mut T {
        unsafe { self.mem.as_mut() }
    }

    pub const fn get_raw(&self) -> NonNull<T> {
        self.mem
    }

    pub fn get_address(&self) -> usize {
        self.mem.addr().into()
    }

    /// Coerce this pointer to a trait object pointer (e.g., GcPtr<dyn Trait>).
    pub fn as_dyn<D: ?Sized>(&self) -> GcPtr<D>
    where
        T: Unsize<D>,
    {
        // SAFETY: The pointer was originally created from a Box<T> and T: Unsize<D>,
        // so the conversion is valid (just like Box<T> -> Box<D>).
        GcPtr {
            mem: self.mem,
            #[cfg(debug_assertions)]
            dealloced: self.dealloced,
        }
    }
}

impl<T> Clone for GcPtr<T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        Self {
            mem: self.mem,
            #[cfg(debug_assertions)]
            dealloced: self.dealloced,
        }
    }
}

impl<T> Copy for GcPtr<T> where T: ?Sized {}

impl<T> Display for GcPtr<T>
where
    T: ?Sized + Display + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { f.write_fmt(format_args!("{:?} -> {}", self.mem, self.mem.as_ref())) }
    }
}

impl<T> Debug for GcPtr<T>
where
    T: ?Sized + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?} -> {:?}", self.mem, self.mem))
    }
}

impl<T> Deref for GcPtr<T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.mem.as_ref() }
    }
}

impl<T> DerefMut for GcPtr<T>
where
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.mem.as_mut() }
    }
}

impl<T> PartialEq for GcPtr<T>
where
    T: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        self.mem == other.mem
    }
}

impl<T> Hashable for GcPtr<T>
where
    T: ?Sized + Hashable,
{
    fn get_hash(&self) -> u32 {
        T::get_hash(self.get())
    }
}

impl AsObjectPtr for GcPtr<Object> {
    fn as_object_ptr(&self) -> GcPtr<Object> {
        self.clone()
    }
}
