use std::{fmt::Debug, mem::MaybeUninit, ptr::NonNull};

use crate::{
    collections::DynArrayIter,
    dbg_println,
    hashing::{GlobalHashMethod, HashMethod, Hashable},
    memory::GC,
};

#[derive(Debug)]
pub struct DynArray<T>
where
    T: Debug,
{
    data: Option<NonNull<T>>,
    count: usize,
    capacity: usize,
    init_value: Option<T>,
}

impl<T> Default for DynArray<T>
where
    T: Debug,
{
    fn default() -> Self {
        Self {
            data: None,
            count: 0,
            capacity: 0,
            init_value: None,
        }
    }
}

/// SAFETY: This is a simple "dynamic array" with minimal safety. It is up to the user to ensure that the data inside is initialized and valid for reading.
#[allow(dead_code)]
impl<T> DynArray<T>
where
    T: Debug,
{
    pub const fn new() -> Self {
        Self {
            data: None,
            count: 0,
            capacity: 0,
            init_value: None,
        }
    }

    pub fn new_with_cap(cap: usize) -> Self {
        let mut me = Self::new();
        if cap > 0 {
            me.grow_array_to(cap);
        }
        me
    }

    pub(crate) const fn get_raw_ptr(&self) -> Option<NonNull<T>> {
        self.data
    }

    /// BE CAREFUL
    pub(super) const unsafe fn set_count(&mut self, new_count: usize) {
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
        // SAFETY: we can unwrap here, since the above statement guarantees that we have data at this point.
        let data = unsafe { self.data.unwrap_unchecked() };

        unsafe {
            let value = data.add(index);

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
        // SAFETY: we can unwrap here, since the above statement guarantees that we have data at this point.
        let data = unsafe { self.data.unwrap_unchecked() };

        unsafe {
            data.add(self.count).write(val);
            self.count += 1;
        }
    }

    pub fn push_ptr(&mut self, val: *const T, count: usize) {
        if self.capacity < self.count + count {
            self.grow_array_to(self.count + count);
        }
        // SAFETY: we can unwrap here, since the above statement guarantees that we have data at this point.
        let data = unsafe { self.data.unwrap_unchecked() };

        unsafe {
            let mut base = data.add(self.count);
            val.copy_to_nonoverlapping(base.as_mut(), count);
            self.count += count
        }
    }

    pub fn push_array(&mut self, other: &DynArray<T>) {
        if let Some(other_data) = other.data {
            self.push_ptr(other_data.as_ptr(), other.count)
        }
    }

    pub fn copy_read(&self, index: usize) -> T {
        debug_assert!(self.data.is_some(), "Tried to read from empty array!");
        // SAFETY: In debug mode we are guaranteed to have data here, in release mode this isn't supposed to happen (lol)
        let data = unsafe { self.data.unwrap_unchecked() };
        unsafe { data.add(index).read() }
    }

    pub fn replace(&mut self, index: usize, new_val: T) {
        debug_assert!(index < self.count, "Index out of bounds: {}", index);
        // SAFETY: we can unwrap here, since the above statement guarantees that we have data at this point.
        let data = unsafe { self.data.unwrap_unchecked() };
        unsafe {
            let value = data.add(index);
            // Drop the old value at index
            std::ptr::drop_in_place(value.as_ptr());
            value.write(new_val)
        }
    }

    /// Gets a reference to the value at the given offset within the capacity of the array.
    pub unsafe fn get_memory_unchecked(&self, offset: usize) -> &T {
        debug_assert!(
            offset < self.capacity,
            "Index out of bounds: {} (count: {})",
            offset,
            self.count
        );
        // SAFETY: we can unwrap here, since the above statement guarantees that we have data at this point.
        let data = unsafe { self.data.unwrap_unchecked() };
        unsafe { data.add(offset).as_ref() }
    }

    /// Gets a mutable reference to the value at the given offset within the capacity of the array.
    pub unsafe fn get_memory_mut_unchecked(&self, offset: usize) -> &mut T {
        debug_assert!(
            offset < self.capacity,
            "Index out of bounds: {} (count: {})",
            offset,
            self.count
        );
        // SAFETY: we can unwrap here, since the above statement guarantees that we have data at this point.
        let data = unsafe { self.data.unwrap_unchecked() };
        unsafe { data.add(offset).as_mut() }
    }

    /// Gets a reference to the value at the given index within the element count of the array.
    pub fn get(&self, index: usize) -> &T {
        debug_assert!(
            index < self.count,
            "Index out of bounds: {} (count: {})",
            index,
            self.count
        );
        // SAFETY: we can unwrap here, since the above statement guarantees that we have data at this point.
        let data = unsafe { self.data.unwrap_unchecked() };
        unsafe { data.add(index).as_ref() }
    }

    /// Gets a mutable reference to the value at the given index within the element count of the array.
    pub fn get_mut(&mut self, index: usize) -> &mut T {
        debug_assert!(
            index < self.count,
            "Index out of bounds: {} (count: {})",
            index,
            self.count
        );
        // SAFETY: we can unwrap here, since the above statement guarantees that we have data at this point.
        let data = unsafe { self.data.unwrap_unchecked() };
        unsafe { data.add(index).as_mut() }
    }

    pub const fn as_slice(&self) -> &[T] {
        if let Some(data) = self.data {
            unsafe { std::slice::from_raw_parts(data.as_ptr(), self.count) }
        } else {
            &[]
        }
    }

    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        if let Some(data) = self.data {
            unsafe { std::slice::from_raw_parts_mut(data.as_ptr(), self.count) }
        } else {
            &mut []
        }
    }

    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, T> {
        if self.data.is_none() {
            return [].iter();
        }

        self.as_slice().iter()
    }

    pub fn iter_mut<'a>(&'a mut self) -> std::slice::IterMut<'a, T> {
        if self.data.is_none() {
            return [].iter_mut();
        }

        self.as_mut_slice().iter_mut()
    }

    /// An iterator over the memory of the array (the whole capacity), which may contain uninitialized values.
    pub fn memory_iter<'a>(&'a self) -> std::slice::Iter<'a, MaybeUninit<T>> {
        if let Some(data) = self.data {
            unsafe { std::slice::from_raw_parts(data.as_ptr().cast(), self.capacity) }.iter()
        } else {
            [].iter()
        }
    }

    /// An iterator over the memory of the array (the whole capacity), which may contain uninitialized values.
    /// This aditionally is a mutable iterator, so be careful.
    pub fn memory_iter_mut<'a>(&'a self) -> std::slice::IterMut<'a, MaybeUninit<T>> {
        if let Some(data) = self.data {
            unsafe {
                std::slice::from_raw_parts_mut(data.as_ptr().cast(), self.capacity).iter_mut()
            }
        } else {
            [].iter_mut()
        }
    }

    fn grow_array_to(&mut self, to: usize) {
        let old_cap = self.capacity;
        self.capacity = to;
        let data = GC.reallocate::<T>(
            self.data.map(|x| x.as_ptr()).unwrap_or_default().cast(),
            old_cap,
            self.capacity,
        );

        if to > 0 {
            let nn: NonNull<T> = unsafe { NonNull::new_unchecked(data.cast()) };
            self.data = Some(nn);
        } else {
            self.data = None;
            return;
        }

        // Copy init value to each new slot
        if let Some(init) = &self.init_value {
            // SAFETY: we can unwrap here, since the above statement guarantees that we have data at this point.
            let data = unsafe { self.data.unwrap_unchecked() };
            let copy_start = old_cap;
            let copy_end = self.capacity;
            for i in copy_start..copy_end {
                unsafe {
                    (init as *const T).copy_to_nonoverlapping(data.as_ptr().add(i), 1);
                }
            }
        }
    }

    pub fn remove_at(&mut self, index: usize) -> T {
        debug_assert!(
            index < self.count,
            "Index out of bounds: {} (count: {})",
            index,
            self.count
        );
        // SAFETY: we can unwrap here, since the above statement guarantees that we have data at this point.
        let data = unsafe { self.data.unwrap_unchecked() };
        unsafe {
            let val_pointer = data.add(index);
            let val = val_pointer.read();
            val_pointer
                .add(1)
                .copy_to(val_pointer, self.count - (index + 1));
            self.count -= 1;
            val
        }
    }

    pub fn pop(&mut self) -> T {
        debug_assert!(self.count > 0, "Cannot pop from an empty array!");
        self.remove_at(self.get_count() - 1)
    }

    pub fn remove_predicate(&mut self, predicate: impl Fn(&T) -> bool) -> Result<T, &'static str> {
        let mut index_to_remove = None;
        for (i, value) in self.iter().enumerate() {
            if predicate(value) {
                index_to_remove = Some(i);
                break;
            }
        }

        if let Some(index) = index_to_remove {
            Ok(self.remove_at(index))
        } else {
            Err("No element found!")
        }
    }
}

impl<T> DynArray<T>
where
    T: std::fmt::Debug + PartialEq,
{
    pub fn remove_value(&mut self, val: T) -> Result<T, &'static str> {
        let mut index_to_remove = None;
        for (i, value) in self.iter().enumerate() {
            if val == *value {
                index_to_remove = Some(i);
                break;
            }
        }

        if let Some(index) = index_to_remove {
            Ok(self.remove_at(index))
        } else {
            Err("No element found!")
        }
    }
}

impl<T> DynArray<T>
where
    T: Debug + Clone,
{
    pub const fn new_with_init(init_value: T) -> Self {
        Self {
            data: None,
            count: 0,
            capacity: 0,
            init_value: Some(init_value),
        }
    }

    pub fn new_with_cap_and_init(cap: usize, init_value: T) -> Self {
        let mut me = Self::new_with_init(init_value);
        me.grow_array_to(cap);
        me
    }
}

impl<T> Clone for DynArray<T>
where
    T: Debug + Clone,
{
    fn clone(&self) -> Self {
        let mut new_array = if let Some(init) = &self.init_value {
            Self::new_with_cap_and_init(self.count, init.clone())
        } else {
            Self::new_with_cap(self.count)
        };

        if let None = self.data {
            return new_array;
        } else if let Some(data) = self.data {
            // SAFETY: We can unwrap here, as the new array is guaranteed to be in the same state as this.
            let new_data = unsafe { new_array.data.unwrap_unchecked() };
            unsafe {
                data.copy_to_nonoverlapping(new_data, self.count);
            }
        }

        new_array.count = self.count;
        new_array
    }
}

// Specialization to make string conversion and raw byte handling easier
impl DynArray<u8> {
    pub unsafe fn read_cast<A>(&self, byte_offset: usize) -> A {
        debug_assert!(
            byte_offset < self.count,
            "Index out of bounds: {} (count: {})",
            byte_offset,
            self.count
        );
        // SAFETY: we can unwrap here, since the above statement guarantees that we have data at this point.
        let data = unsafe { self.data.unwrap_unchecked() };

        // First offset by n-bytes and then cast
        unsafe { data.add(byte_offset).cast::<A>().read() }
    }

    pub fn from_str(str: &str) -> Self {
        let mut me = Self::new_with_cap(str.len());
        me.push_ptr(str.as_ptr(), str.len());
        me
    }

    pub const fn as_str(&self) -> &str {
        if let Some(data) = self.data {
            unsafe { std::str::from_raw_parts(data.as_ptr(), self.count) }
        } else {
            ""
        }
    }
}

impl Hashable for DynArray<u8> {
    fn get_hash(&self) -> u32 {
        if let Some(data) = self.data {
            unsafe {
                GlobalHashMethod::hash(std::slice::from_raw_parts_mut(data.as_ptr(), self.count))
            }
        } else {
            0
        }
    }
}

impl PartialEq for DynArray<u8> {
    fn eq(&self, other: &Self) -> bool {
        self.get_hash() == other.get_hash()
    }
}

impl<T: std::fmt::Debug> Drop for DynArray<T> {
    fn drop(&mut self) {
        if self.data.is_none() {
            return;
        }
        // SAFETY: we can unwrap here, since the above statement guarantees that we have data at this point.
        let data = unsafe { self.data.unwrap_unchecked() };

        dbg_println!("DEBUG DYNARRAY DROP: {:?}", self.data);

        unsafe {
            if self.init_value.is_some() {
                std::ptr::drop_in_place(self.as_mut_slice());
            } else {
                // If we don't have an init value, we only drop up until self.count which is guaranteed to be initialized.

                let values = std::slice::from_raw_parts_mut(data.as_ptr(), self.count);
                std::ptr::drop_in_place(values);
            }
        }

        // Dealloc the data.
        GC.reallocate::<T>(data.as_ptr().cast(), self.capacity, 0);

        self.data = None;
        self.capacity = 0;
        self.count = 0;
    }
}

impl<T> IntoIterator for DynArray<T>
where
    T: Debug,
{
    type Item = T;
    type IntoIter = DynArrayIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        DynArrayIter::new(self)
    }
}
