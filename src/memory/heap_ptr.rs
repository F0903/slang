use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use super::Dealloc;
use crate::hashing::Hashable;

// A manual version of Box<T> that REQUIRES YOU TO MANUALLY CALL DEALLOC TO FREE MEMORY
// This is useful for heap allocated objects that require multiple references to the same object and lowest overhead (thus not using Rc<RefCell<T>> or similar).
pub struct HeapPtr<T> {
    mem: NonNull<T>,
    #[cfg(debug_assertions)]
    dealloced: bool,
}

impl<T> HeapPtr<T>
where
    T: Debug,
{
    pub fn alloc(obj: T) -> Self {
        // Using Box::leak is more efficient than manually allocating due to some internal Rust optimizations. ;
        Self {
            // SAFETY: This is guaranteed to be non-null, as we are literally creating the Box right here.
            mem: unsafe { NonNull::new_unchecked(Box::leak(Box::new(obj))) },
            #[cfg(debug_assertions)]
            dealloced: true,
        }
    }

    pub const fn from_raw(ptr: NonNull<T>) -> Self {
        Self {
            mem: ptr,
            #[cfg(debug_assertions)]
            dealloced: true,
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

    /// This will take ownership of the object and return it.
    /// This makes the underlying value be exposed to the normal drop rules.
    pub fn take(self) -> T {
        let val = unsafe { *Box::from_raw(self.mem.as_ptr()) };
        val
    }

    pub fn read(&self) -> T {
        unsafe { self.mem.read() }
    }
}

impl<T> Dealloc for HeapPtr<T>
where
    T: Debug,
{
    // Despite specilization being unsound, I do not believe this case will cause UB
    default fn dealloc(&mut self) {
        #[cfg(debug_assertions)]
        {
            assert!(!self.dealloced, "Double free detected!");
            println!("HEAPPTR DEALLOC (INNER DROP): {:?}", self);
            self.dealloced = true;
        }
        unsafe {
            if std::mem::needs_drop::<T>() {
                drop(Box::from_raw(self.mem.as_ptr()));
            }
        }
    }
}

impl<T> Dealloc for HeapPtr<T>
where
    T: Dealloc + Debug,
{
    fn dealloc(&mut self) {
        #[cfg(debug_assertions)]
        {
            assert!(!self.dealloced, "Double free detected!");
            println!("HEAPPTR DEALLOC (INNER DEALLOC): {:?}", self);
            self.dealloced = true;
        }
        self.take().dealloc();
    }
}

impl<T> Clone for HeapPtr<T> {
    fn clone(&self) -> Self {
        Self {
            mem: self.mem,
            #[cfg(debug_assertions)]
            dealloced: self.dealloced,
        }
    }
}

impl<T> Copy for HeapPtr<T> {}

impl<T> Display for HeapPtr<T>
where
    T: Display + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { f.write_fmt(format_args!("{:?} -> {}", self.mem, self.mem.as_ref())) }
    }
}

impl<T> Debug for HeapPtr<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?} -> {:?}", self.mem, self.mem))
    }
}

impl<T> Deref for HeapPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.mem.as_ref() }
    }
}

impl<T> DerefMut for HeapPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.mem.as_mut() }
    }
}

impl<T> PartialEq for HeapPtr<T>
where
    T: PartialEq + Debug,
{
    fn eq(&self, other: &Self) -> bool {
        self.mem == other.mem
    }
}

impl<T> Hashable for HeapPtr<T>
where
    T: Debug + Hashable,
{
    fn get_hash(&self) -> u32 {
        T::get_hash(self.get())
    }
}
