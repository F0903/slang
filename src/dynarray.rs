use {crate::memory::reallocate, std::ptr::null_mut};

pub struct DynArray<T> {
    data: *mut T,
    count: usize,
    capacity: usize,
}

impl<T> DynArray<T> {
    pub fn new() -> Self {
        Self {
            data: null_mut(),
            count: 0,
            capacity: 0,
        }
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

    pub const fn get_count(&self) -> usize {
        self.count
    }

    pub const fn get_capacity(&self) -> usize {
        self.capacity
    }

    pub const fn grow_capacity(&self) -> usize {
        const MIN_CAP: usize = 16;
        const GROW_FACTOR: usize = 2;
        if self.capacity < MIN_CAP {
            MIN_CAP
        } else {
            self.capacity * GROW_FACTOR
        }
    }

    pub fn grow_array_to(&mut self, to: usize) {
        let old_cap = self.capacity;
        self.capacity = to;
        self.data = reallocate::<T>(self.data.cast(), old_cap, self.capacity).cast();
    }

    pub fn grow_array(&mut self) {
        self.grow_array_to(self.grow_capacity())
    }

    pub fn insert(&mut self, index: usize, val: T) {
        if self.capacity < index + 1 {
            self.grow_array_to(index + 1);
        }

        unsafe {
            self.data.add(index).write(val);
        }
    }

    pub fn write(&mut self, val: T) {
        if self.capacity < self.count + 1 {
            self.grow_array();
        }

        unsafe {
            self.data.add(self.count).write(val);
            self.count += 1;
        }
    }

    pub fn write_ptr(&mut self, val: *const T, count: usize) {
        if self.capacity < self.count + count {
            self.grow_array();
        }

        unsafe {
            let base = self.data.add(self.count);
            std::ptr::copy_nonoverlapping(val, base, count);
            self.count += count
        }
    }

    /// RETURNS VALUE POINTING INTO THE ARRAY
    pub const fn read(&self, offset: usize) -> T {
        unsafe { self.data.add(offset).read() }
    }

    //TODO: mark as const when const as_ref() is stable
    pub fn get(&self, offset: usize) -> Option<&T> {
        unsafe { self.data.add(offset).as_ref() }
    }
}

impl<T> DynArray<T>
where
    T: Clone,
{
    pub fn read_copy(&self, offset: usize) -> T {
        self.read(offset).clone()
    }
}

impl DynArray<u8> {
    pub const fn read_cast<A>(&self, offset: usize) -> A {
        unsafe { self.data.cast::<A>().add(offset).read() }
    }
}

impl<T> Drop for DynArray<T> {
    fn drop(&mut self) {
        self.data = reallocate::<T>(self.data.cast(), self.capacity, 0).cast();
        self.capacity = 0;
        self.count = 0;
    }
}
