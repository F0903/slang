use std::{mem::MaybeUninit, ptr::null_mut};

use super::{borrowed_ptr_iter::BorrowedIter, owned_ptr_iter::OwnedPtrIter};
use crate::{dbg_println, memory::reallocate};

trait GrowArray {
    fn grow_array_to(&mut self, to: usize);
}

#[derive(Debug)]
pub struct DynArray<T: std::fmt::Debug> {
    data: *mut T,
    count: usize,
    capacity: usize,
    init_value: Option<T>,
}

impl<T: std::fmt::Debug> Default for DynArray<T> {
    fn default() -> Self {
        Self {
            data: null_mut(),
            count: 0,
            capacity: 0,
            init_value: None,
        }
    }
}

/// SAFETY: This is a simple "dynamic array" with minimal safety. It is up to the user to ensure that the data inside is initialized and valid for reading.
impl<T: std::fmt::Debug> DynArray<T> {
    pub const fn new(init_value: Option<T>) -> Self {
        Self {
            data: null_mut(),
            count: 0,
            capacity: 0,
            init_value,
        }
    }

    pub fn new_with_cap(cap: usize, init_value: Option<T>) -> Self {
        let mut me = Self::new(init_value);
        me.grow_array_to(cap);
        me
    }

    pub(crate) const fn get_raw_ptr(&self) -> *mut T {
        self.data
    }

    /// ASSUMES CALLER FREES OLD DATA
    pub(crate) fn set_backing_data(&mut self, new_data: *mut T, new_count: usize, new_cap: usize) {
        self.data = new_data;
        self.count = new_count;
        self.capacity = new_cap;
    }

    /// BE CAREFUL
    pub(crate) const fn set_count(&mut self, new_count: usize) {
        self.count = new_count;
    }

    pub const fn get_count(&self) -> usize {
        self.count
    }

    pub const fn get_capacity(&self) -> usize {
        self.capacity
    }

    pub const fn next_growth_capacity(&self) -> usize {
        const MIN_CAP: usize = 16;
        const GROW_FACTOR: usize = 2;
        if self.capacity < MIN_CAP {
            MIN_CAP
        } else {
            self.capacity * GROW_FACTOR
        }
    }

    pub fn grow_array(&mut self) {
        self.grow_array_to(self.next_growth_capacity())
    }

    pub fn insert(&mut self, index: usize, val: T) {
        if self.capacity < index + 1 {
            self.grow_array_to(index + 1);
        }

        unsafe {
            self.data.add(index).write(val);
        }
    }

    pub fn push(&mut self, val: T) {
        if self.capacity < self.count + 1 {
            self.grow_array();
        }

        unsafe {
            self.data.add(self.count).write(val);
            self.count += 1;
        }
    }

    pub fn push_ptr(&mut self, val: *const T, count: usize) {
        if self.capacity < self.count + count {
            self.grow_array();
        }

        unsafe {
            let base = self.data.add(self.count);
            std::ptr::copy_nonoverlapping(val, base, count);
            self.count += count
        }
    }

    pub fn push_array(&mut self, other: &DynArray<T>) {
        self.push_ptr(other.data, other.count)
    }

    pub fn copy_read(&self, index: usize) -> T {
        unsafe { self.data.add(index).read() }
    }

    pub fn read(&self, index: usize) -> &T {
        unsafe { &*self.data.add(index) }
    }

    pub fn replace(&self, index: usize, new_val: T) {
        unsafe { self.data.add(index).write(new_val) }
    }

    pub const fn get(&self, offset: usize) -> Option<&T> {
        unsafe { self.data.add(offset).as_ref() }
    }

    pub const fn get_unchecked(&self, offset: usize) -> &T {
        unsafe { self.data.add(offset).as_ref_unchecked() }
    }

    pub const fn get_mut(&mut self, offset: usize) -> Option<&mut T> {
        unsafe { self.data.add(offset).as_mut() }
    }

    pub const fn get_mut_unchecked(&mut self, offset: usize) -> &mut T {
        unsafe { self.data.add(offset).as_mut_unchecked() }
    }

    pub const fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.data, self.count) }
    }

    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.data, self.count) }
    }

    pub const fn iter<'a>(&'a self) -> BorrowedIter<'a, T> {
        if self.data.is_null() {
            return BorrowedIter::new(&[]);
        }

        BorrowedIter::new(self.as_slice())
    }

    /// An iterator over the memory of the array (the whole capacity), which may contain uninitialized values.
    pub const fn memory_iter<'a>(&'a self) -> BorrowedIter<'a, MaybeUninit<T>> {
        if self.data.is_null() {
            return BorrowedIter::new(&[]);
        }

        BorrowedIter::new(unsafe { std::slice::from_raw_parts(self.data.cast(), self.capacity) })
    }
}

impl<T: std::fmt::Debug> Clone for DynArray<T> {
    fn clone(&self) -> Self {
        let mut new_array = Self::new_with_cap(self.count, None);
        if self.data.is_null() {
            return new_array;
        }

        unsafe {
            std::ptr::copy_nonoverlapping(self.data, new_array.data, self.count);
        }
        new_array.count = self.count;
        new_array
    }
}

impl<T: std::fmt::Debug> GrowArray for DynArray<T> {
    fn grow_array_to(&mut self, to: usize) {
        let old_cap = self.capacity;
        self.capacity = to;
        self.data = reallocate::<T>(self.data.cast(), old_cap, self.capacity).cast();

        // Copy init value to each new slot
        // Yes, I know this is not the safest way, but it is faster that cloning each and easier.
        if let Some(init) = &self.init_value {
            let copy_start = old_cap;
            let copy_end = self.capacity;
            for i in copy_start..copy_end {
                unsafe {
                    (init as *const T).copy_to_nonoverlapping(self.data.add(i), 1);
                }
            }
        }
    }
}

// Specialization to make string conversion and raw byte handling easier
impl DynArray<u8> {
    pub fn read_cast<A>(&self, byte_offset: usize) -> A {
        // First offset by n-bytes and then cast
        unsafe { self.data.add(byte_offset).cast::<A>().read() }
    }

    pub fn from_str(str: &str) -> Self {
        let mut me = Self::new_with_cap(str.len(), None);
        me.push_ptr(str.as_ptr(), str.len());
        me
    }

    pub const fn as_str(&self) -> &str {
        unsafe { std::str::from_raw_parts(self.data, self.count) }
    }
}

impl<T: std::fmt::Debug> Drop for DynArray<T> {
    fn drop(&mut self) {
        if self.data.is_null() {
            return;
        }

        dbg_println!("DEBUG DYNARRAY DROP: {:?}", self);
        unsafe {
            std::ptr::drop_in_place(self.as_mut_slice());
        }
        self.data = reallocate::<T>(self.data.cast(), self.capacity, 0).cast();
        self.capacity = 0;
        self.count = 0;
    }
}

impl<T: std::fmt::Debug> IntoIterator for DynArray<T> {
    type Item = T;
    type IntoIter = OwnedPtrIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        OwnedPtrIter::new(self.data, self.count)
    }
}
