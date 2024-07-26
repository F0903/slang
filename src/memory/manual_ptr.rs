use super::{allocate, free, Dealloc};
use std::{
    fmt::{Debug, Display},
    ptr::{self, null_mut},
};

#[derive(PartialEq, PartialOrd, Debug)]
pub struct ManualPtr<T> {
    mem: *mut T,
}

impl<T> ManualPtr<T>
where
    T: Debug,
{
    pub fn alloc(obj: T) -> Self {
        println!("DEBUG MANUALPTR: {:?}", obj);
        unsafe {
            let mem = allocate::<T>();
            ptr::write(mem, obj);
            Self { mem }
        }
    }

    pub const fn from_raw(ptr: *mut T) -> Self {
        Self { mem: ptr }
    }

    //TODO: mark const when stable
    pub fn get(&self) -> &T {
        unsafe { &(*self.mem) }
    }

    pub const fn get_raw(&self) -> *mut T {
        self.mem
    }

    //TODO: mark const when stable
    /// THIS WILL CAUSE A LEAK IF NOT CAREFUL
    pub fn take(self) -> T {
        unsafe { self.mem.read() }
    }

    //TODO: mark const when stable
    pub fn is_null(&self) -> bool {
        self.mem.is_null()
    }

    pub const fn null() -> Self {
        Self { mem: null_mut() }
    }
}

impl<T> Dealloc for ManualPtr<T> {
    fn dealloc(&mut self) {
        free(self.mem);
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
