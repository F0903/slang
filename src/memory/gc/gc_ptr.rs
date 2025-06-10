use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{
    hashing::Hashable,
    memory::{DropDealloc, GC},
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
    T: Sized + Debug,
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

    pub fn dealloc(&mut self) {
        #[cfg(debug_assertions)]
        {
            assert!(!self.dealloced, "Double free detected!");
            println!("GCPTR DEALLOC: {:?}", self);
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
}

impl<T> GcPtr<T>
where
    T: Debug,
{
    /// This will take ownership of the object and return it.
    /// This makes the underlying value be exposed to the normal drop rules.
    pub fn take(self) -> T {
        let val = unsafe { *Box::from_raw_in(self.mem.as_ptr(), &GC) };
        val
    }
}

impl<T> GcPtr<T>
where
    T: ?Sized + Debug,
{
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
}

impl<T> Clone for GcPtr<T> {
    fn clone(&self) -> Self {
        Self {
            mem: self.mem,
            #[cfg(debug_assertions)]
            dealloced: self.dealloced,
        }
    }
}

impl<T> Copy for GcPtr<T> {}

impl<T> Display for GcPtr<T>
where
    T: Display + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { f.write_fmt(format_args!("{:?} -> {}", self.mem, self.mem.as_ref())) }
    }
}

impl<T> Debug for GcPtr<T>
where
    T: Debug,
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

impl<T> DerefMut for GcPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.mem.as_mut() }
    }
}

impl<T> PartialEq for GcPtr<T>
where
    T: PartialEq + Debug,
{
    fn eq(&self, other: &Self) -> bool {
        self.mem == other.mem
    }
}

impl<T> Hashable for GcPtr<T>
where
    T: Debug + Hashable,
{
    fn get_hash(&self) -> u32 {
        T::get_hash(self.get())
    }
}
