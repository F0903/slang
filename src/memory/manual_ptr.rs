use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    ptr::{self, null_mut},
};

use super::dealloc::Dealloc;

// A manual version of Box<T> that REQUIRES YOU TO MANUALLY CALL DEALLOC TO FREE MEMORY
// This is useful for heap allocated objects that require multiple references to the same object and lowest overhead (thus not using Rc<RefCell<T>> or similar).
#[derive(PartialEq, PartialOrd, Debug)]
pub struct ManualPtr<T> {
    mem: *mut T,
}

impl<T> ManualPtr<T>
where
    // temp
    T: Debug,
{
    pub fn alloc(obj: T) -> Self {
        println!("DEBUG MANUALPTR: {:?}", obj);
        // Using Box::leak is more efficient than manually allocating due to some internal Rust optimizations.
        let mem = Box::leak(Box::new(obj));
        Self { mem }
    }

    pub const fn from_raw(ptr: *mut T) -> Self {
        Self { mem: ptr }
    }

    pub const fn get(&self) -> &T {
        unsafe { &(*self.mem) }
    }

    pub const fn get_mut(&self) -> &mut T {
        unsafe { &mut (*self.mem) }
    }

    pub const fn get_raw(&self) -> *mut T {
        self.mem
    }

    /// THIS WILL CAUSE A LEAK IF NOT CAREFUL
    pub const fn take(self) -> T {
        unsafe { self.mem.read() }
    }

    pub const fn is_null(&self) -> bool {
        self.mem.is_null()
    }

    pub const fn null() -> Self {
        Self { mem: null_mut() }
    }
}

impl<T> Dealloc for ManualPtr<T>
where
    T: Debug,
{
    default fn dealloc(&mut self) {
        if self.is_null() {
            return;
        }
        unsafe {
            drop(Box::from_raw(self.mem));
        }
        self.mem = ptr::null_mut();
    }
}

impl<T> Dealloc for ManualPtr<T>
where
    T: Dealloc + Debug,
{
    fn dealloc(&mut self) {
        if self.is_null() {
            return;
        }
        self.take().dealloc(); // Run the dealloc method on the object we are pointing to first.
        unsafe {
            drop(Box::from_raw(self.mem));
        }
        self.mem = ptr::null_mut();
    }
}

impl<T> Clone for ManualPtr<T> {
    fn clone(&self) -> Self {
        Self { mem: self.mem }
    }
}

impl<T> Copy for ManualPtr<T> {}

impl<T> Display for ManualPtr<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { f.write_fmt(format_args!("{}", *self.mem)) }
    }
}

impl<T> Deref for ManualPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mem }
    }
}

impl<T> DerefMut for ManualPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mem }
    }
}
