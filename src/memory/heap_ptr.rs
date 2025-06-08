use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    ptr::{self, null_mut},
};

use super::Dealloc;
use crate::{dbg_println, hashing::Hashable};

// A manual version of Box<T> that REQUIRES YOU TO MANUALLY CALL DEALLOC TO FREE MEMORY
// This is useful for heap allocated objects that require multiple references to the same object and lowest overhead (thus not using Rc<RefCell<T>> or similar).
pub struct HeapPtr<T> {
    mem: *mut T,
}

impl<T> HeapPtr<T>
where
    T: Debug,
{
    pub fn alloc(obj: T) -> Self {
        // Using Box::leak is more efficient than manually allocating due to some internal Rust optimizations.
        let mem = Box::leak(Box::new(obj));
        Self { mem }
    }

    pub const fn from_raw(ptr: *mut T) -> Self {
        Self { mem: ptr }
    }

    pub const fn get(&self) -> &T {
        debug_assert!(self.is_not_null(), "Cannot get value from null HeapPtr!");
        unsafe { &(*self.mem) }
    }

    pub const fn get_mut(&self) -> &mut T {
        debug_assert!(self.is_not_null(), "Cannot get value from null HeapPtr!");
        unsafe { &mut (*self.mem) }
    }

    pub const fn get_raw(&self) -> *mut T {
        self.mem
    }

    /// This will take ownership of the object and return it.
    /// This makes the underlying value be exposed to the normal drop rules.
    pub fn take(mut self) -> T {
        debug_assert!(self.is_not_null(), "Cannot take value from null HeapPtr!");
        let val = unsafe { *Box::from_raw(self.mem) };
        self.mem = null_mut();
        val
    }

    pub fn read(&self) -> T {
        debug_assert!(self.is_not_null(), "Cannot read value from null HeapPtr!");
        unsafe { self.mem.read() }
    }

    pub const fn is_null(&self) -> bool {
        self.mem.is_null()
    }

    pub const fn is_not_null(&self) -> bool {
        !self.is_null()
    }

    pub const fn null() -> Self {
        Self { mem: null_mut() }
    }
}

impl<T> Dealloc for HeapPtr<T>
where
    T: Debug,
{
    // Despite specilization being unsound, I do not believe this case will cause UB
    default fn dealloc(&mut self) {
        if self.is_null() {
            return;
        }

        dbg_println!("HEAPPTR DEALLOC (INNER DROP): {:?}", self);
        unsafe {
            if std::mem::needs_drop::<T>() {
                drop(Box::from_raw(self.mem));
            }
            self.mem = ptr::null_mut();
        }
    }
}

impl<T> Dealloc for HeapPtr<T>
where
    T: Dealloc + Debug,
{
    fn dealloc(&mut self) {
        if self.is_null() {
            return;
        }

        dbg_println!("HEAPPTR DEALLOC (INNER DEALLOC): {:?}", self);
        self.take().dealloc();
        self.mem = self::null_mut();
    }
}

impl<T> Clone for HeapPtr<T> {
    fn clone(&self) -> Self {
        Self { mem: self.mem }
    }
}

impl<T> Copy for HeapPtr<T> {}

impl<T> Display for HeapPtr<T>
where
    T: Display + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            return f.write_str("null");
        }
        unsafe { f.write_fmt(format_args!("{:?} -> {}", self.mem, *self.mem)) }
    }
}

impl<T> Debug for HeapPtr<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            return f.write_str("null");
        }
        unsafe { f.write_fmt(format_args!("{:?} -> {:?}", self.mem, *self.mem)) }
    }
}

impl<T> Deref for HeapPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mem }
    }
}

impl<T> DerefMut for HeapPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mem }
    }
}

impl<T> PartialEq for HeapPtr<T>
where
    T: PartialEq + Debug,
{
    fn eq(&self, other: &Self) -> bool {
        if self.is_null() && other.is_null() {
            return true;
        }
        if self.is_null() || other.is_null() {
            return false;
        }
        unsafe { &*self.mem == &*other.mem }
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
