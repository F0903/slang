use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct ObjectRef<T> {
    ///SAFETY: This is guaranteed to live until GC'd, at which point there is no references to the object (hence it was GC'd)
    ptr: *const T,
}

impl<T> ObjectRef<T> {
    pub(super) fn new(ptr: *const T) -> Self {
        Self { ptr }
    }

    pub const fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref_unchecked() }
    }

    pub const fn as_mut(&mut self) -> &mut T {
        // Casting from const to mut here is obviously extra unsafe, but we are riding wild out here.
        unsafe { (self.ptr as *mut T).as_mut_unchecked() }
    }

    pub fn addr_gt_addr<A>(&self, other: *const A) -> bool {
        (self.ptr as usize) > (other as usize)
    }

    pub fn addr_eq_addr<A>(&self, other: *const A) -> bool {
        (self.ptr as usize) == (other as usize)
    }

    pub fn addr_lt_addr<A>(&self, other: *const A) -> bool {
        (self.ptr as usize) < (other as usize)
    }
}

impl<T> Clone for ObjectRef<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T> Copy for ObjectRef<T> {}

impl<T> Deref for ObjectRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for ObjectRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
