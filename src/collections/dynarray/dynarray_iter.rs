use std::fmt::Debug;

use crate::collections::DynArray;

pub struct DynArrayIter<T>
where
    T: Debug,
{
    inner: DynArray<T>,
    index: usize,
}

impl<T> DynArrayIter<T>
where
    T: Debug,
{
    pub const fn new(inner: DynArray<T>) -> Self {
        Self { inner, index: 0 }
    }
}

impl<T> Iterator for DynArrayIter<T>
where
    T: Debug,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.inner.get_count() {
            return None;
        }

        let ptr = self.inner.get_raw_ptr();
        if let Some(ptr) = ptr {
            // SAFETY: we just checked that index is less than len, so we are in-bounds
            let val = unsafe { ptr.add(self.index).read() };
            self.index += 1;
            Some(val)
        } else {
            None
        }
    }
}
