use std::{
    alloc::Allocator,
    fmt::{Debug, Display},
    marker::Unsize,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{hashing::Hashable, memory::DropDealloc};

/// A manual version of Box that allocates with the GC.
/// SAFETY: This is a wrapper around a NonNull pointer, cloning will only clone the pointer, and not the underlying data.
pub struct HeapPtr<T: ?Sized> {
    mem: NonNull<T>,
    #[cfg(debug_assertions)]
    dealloced: bool,
}

#[allow(dead_code)]
impl<T> HeapPtr<T>
where
    T: Sized,
{
    #[inline]
    pub fn alloc(obj: T) -> Self {
        // Using Box::leak is more efficient than manually allocating due to some internal Rust optimizations.
        Self {
            // SAFETY: This is guaranteed to be non-null, as we are literally creating the Box right here.
            mem: unsafe { NonNull::new_unchecked(Box::leak(Box::new(obj))) },
            #[cfg(debug_assertions)]
            dealloced: false,
        }
    }

    /// This will take ownership of the object and return it.
    /// This makes the underlying value be exposed to the normal drop rules.
    #[inline]
    pub fn take(self) -> T {
        // SAFETY: This is always guaranteed to be non-null.
        let val = unsafe { *Box::from_raw(self.mem.as_ptr()) };
        val
    }

    #[inline]
    pub fn alloc_in(obj: T, alloc: impl Allocator) -> Self {
        // Using Box::leak is more efficient than manually allocating due to some internal Rust optimizations.
        Self {
            // SAFETY: This is guaranteed to be non-null, as we are literally creating the Box right here.
            mem: unsafe { NonNull::new_unchecked(Box::leak(Box::new_in(obj, &alloc))) },
            #[cfg(debug_assertions)]
            dealloced: false,
        }
    }

    /// This will take ownership of the object and return it.
    /// This makes the underlying value be exposed to the normal drop rules.
    #[inline]
    pub fn take_in(self, alloc: impl Allocator) -> T {
        // SAFETY: This is always guaranteed to be non-null.
        let val = unsafe { *Box::from_raw_in(self.mem.as_ptr(), &alloc) };
        val
    }
}

impl<T> HeapPtr<T>
where
    T: ?Sized,
{
    pub fn dealloc(&mut self) {
        #[cfg(debug_assertions)]
        {
            assert!(!self.dealloced, "Double free detected!");
            self.dealloced = true;
        }
        // SAFETY: We are deallocating the memory that we own, and we are guaranteed to have a valid pointer.
        unsafe {
            if std::mem::needs_drop::<T>() {
                drop(Box::from_raw(self.mem.as_ptr()));
            }
        }
    }

    #[inline]
    pub fn dealloc_on_drop(self) -> DropDealloc<T> {
        DropDealloc::new(self)
    }

    #[inline]
    pub fn take_box(boxed: Box<T>) -> Self {
        Self {
            // SAFETY: This is guaranteed to be non-null, as we are literally creating the Box right here.
            mem: unsafe { NonNull::new_unchecked(Box::leak(boxed)) },
            #[cfg(debug_assertions)]
            dealloced: false,
        }
    }

    #[inline]
    pub const fn from_raw(ptr: NonNull<T>) -> Self {
        Self {
            mem: ptr,
            #[cfg(debug_assertions)]
            dealloced: false,
        }
    }

    #[inline]
    pub const fn get(&self) -> &T {
        // SAFETY: self.mem is always guaranteed to be non-null and valid.
        unsafe { self.mem.as_ref() }
    }

    #[inline]
    pub const fn get_mut(&mut self) -> &mut T {
        // SAFETY: self.mem is always guaranteed to be non-null and valid.
        unsafe { self.mem.as_mut() }
    }

    #[inline]
    pub const fn get_raw(&self) -> NonNull<T> {
        self.mem
    }

    #[inline]
    pub fn get_address(&self) -> usize {
        self.mem.addr().into()
    }

    /// Coerce this pointer to a trait object pointer (e.g., GcPtr<dyn Trait>).
    #[inline]
    pub fn as_dyn<D: ?Sized>(&self) -> HeapPtr<D>
    where
        T: Unsize<D>,
    {
        // SAFETY: The pointer was originally created from a Box<T> and T: Unsize<D>,
        // so the conversion is valid (just like Box<T> -> Box<D>).
        HeapPtr {
            mem: self.mem,
            #[cfg(debug_assertions)]
            dealloced: self.dealloced,
        }
    }
}

impl<T> Clone for HeapPtr<T>
where
    T: ?Sized,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            mem: self.mem,
            #[cfg(debug_assertions)]
            dealloced: self.dealloced,
        }
    }
}

impl<T> Copy for HeapPtr<T> where T: ?Sized {}

impl<T> Display for HeapPtr<T>
where
    T: ?Sized + Display + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SAFETY: self.mem is always guaranteed to be non-null and valid.
        unsafe { f.write_fmt(format_args!("{:?} -> {}", self.mem, self.mem.as_ref())) }
    }
}

impl<T> Debug for HeapPtr<T>
where
    T: ?Sized + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?} -> {:?}", self.mem, self.mem))
    }
}

impl<T> Deref for HeapPtr<T>
where
    T: ?Sized,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: self.mem is always guaranteed to be non-null and valid.
        unsafe { self.mem.as_ref() }
    }
}

impl<T> DerefMut for HeapPtr<T>
where
    T: ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: self.mem is always guaranteed to be non-null and valid.
        unsafe { self.mem.as_mut() }
    }
}

impl<T> PartialEq for HeapPtr<T>
where
    T: ?Sized,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // SAFETY: self.mem is always guaranteed to be non-null and valid.
        std::ptr::addr_eq(self.mem.as_ptr(), other.mem.as_ptr())
    }
}

impl<T> Hashable for HeapPtr<T>
where
    T: ?Sized + Hashable,
{
    #[inline]
    fn get_hash(&self) -> u32 {
        T::get_hash(self.get())
    }
}
