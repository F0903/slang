use std::{
    fmt::{Debug, Display},
    ptr::{self, null_mut},
};

use super::dealloc::Dealloc;

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
        let mem = Box::leak(Box::new(obj));
        Self { mem }
    }

    pub const fn from_raw(ptr: *mut T) -> Self {
        Self { mem: ptr }
    }

    pub const fn get(&self) -> &T {
        unsafe { &(*self.mem) }
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
