use std::{mem::MaybeUninit, ptr::null_mut};

use super::owned_iter::OwnedIter;
use crate::{dbg_println, memory::reallocate};

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

    /// BE CAREFUL
    pub(super) const fn set_count(&mut self, new_count: usize) {
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
            let value = self.data.add(index);

            if index < self.count {
                // Shift everything over once for our element to be inserted.
                value.copy_to(value.add(1), self.count - index);
            }

            value.write(val);
            self.count += 1;
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
            val.copy_to_nonoverlapping(base, count);
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
        debug_assert!(index < self.count, "Index out of bounds: {}", index);
        unsafe {
            let value = self.data.add(index);
            // Drop the old value at index
            std::ptr::drop_in_place(value);
            value.write(new_val)
        }
    }

    /// Gets a reference to the value at the given offset within the capacity of the array.
    pub fn get_memory(&self, offset: usize) -> &T {
        debug_assert!(
            offset < self.capacity,
            "Index out of bounds: {} (count: {})",
            offset,
            self.count
        );
        unsafe { self.data.add(offset).as_ref_unchecked() }
    }

    /// Gets a mutable reference to the value at the given offset within the capacity of the array.
    pub fn get_memory_mut(&self, offset: usize) -> &mut T {
        debug_assert!(
            offset < self.capacity,
            "Index out of bounds: {} (count: {})",
            offset,
            self.count
        );
        unsafe { self.data.add(offset).as_mut_unchecked() }
    }

    /// Gets a reference to the value at the given offset within the element count of the array.
    pub fn get(&self, offset: usize) -> &T {
        debug_assert!(
            offset < self.count,
            "Index out of bounds: {} (count: {})",
            offset,
            self.count
        );
        unsafe { self.data.add(offset).as_ref_unchecked() }
    }

    pub fn get_mut(&mut self, offset: usize) -> &mut T {
        debug_assert!(
            offset < self.count,
            "Index out of bounds: {} (count: {})",
            offset,
            self.count
        );
        unsafe { self.data.add(offset).as_mut_unchecked() }
    }

    pub const fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.data, self.count) }
    }

    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.data, self.count) }
    }

    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, T> {
        if self.data.is_null() {
            return [].iter();
        }

        self.as_slice().iter()
    }

    /// An iterator over the memory of the array (the whole capacity), which may contain uninitialized values.
    pub fn memory_iter<'a>(&'a self) -> std::slice::Iter<'a, MaybeUninit<T>> {
        if self.data.is_null() {
            return [].iter();
        }

        unsafe { std::slice::from_raw_parts(self.data.cast(), self.capacity) }.iter()
    }

    /// An iterator over the memory of the array (the whole capacity), which may contain uninitialized values.
    /// This aditionally is a mutable iterator, so be careful.
    pub fn memory_iter_mut<'a>(&'a self) -> std::slice::IterMut<'a, MaybeUninit<T>> {
        if self.data.is_null() {
            return [].iter_mut();
        }

        unsafe { std::slice::from_raw_parts_mut(self.data.cast(), self.capacity).iter_mut() }
    }

    fn grow_array_to(&mut self, to: usize) {
        let old_cap = self.capacity;
        self.capacity = to;
        self.data = reallocate::<T>(self.data.cast(), old_cap, self.capacity).cast();

        // Copy init value to each new slot
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

impl<T: std::fmt::Debug> Clone for DynArray<T> {
    fn clone(&self) -> Self {
        let mut new_array = Self::new_with_cap(self.count, None);
        if self.data.is_null() {
            return new_array;
        }

        unsafe {
            self.data.copy_to_nonoverlapping(new_array.data, self.count);
        }
        new_array.count = self.count;
        new_array
    }
}

// Specialization to make string conversion and raw byte handling easier
impl DynArray<u8> {
    pub fn read_cast<A>(&self, byte_offset: usize) -> A {
        debug_assert!(
            byte_offset < self.count,
            "Index out of bounds: {} (count: {})",
            byte_offset,
            self.count
        );

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
            if self.init_value.is_some() {
                std::ptr::drop_in_place(self.as_mut_slice());
            } else {
                // If we don't have an init value, we only drop up until self.count which is guaranteed to be initialized.
                let values = std::slice::from_raw_parts_mut(self.data, self.count);
                std::ptr::drop_in_place(values);
            }
        }

        self.data = reallocate::<T>(self.data.cast(), self.capacity, 0).cast();
        self.capacity = 0;
        self.count = 0;
    }
}

impl<T: std::fmt::Debug> IntoIterator for DynArray<T> {
    type Item = T;
    type IntoIter = OwnedIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        OwnedIter::new(self.data, self.count)
    }
}
