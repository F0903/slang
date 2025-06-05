/// This struct implements an iterator that returns unbounded lifetimes from its pointer.
/// In other words, you must ensure that every element this points to is valid for the iterator.
pub struct UnsafePtrIter<T> {
    data: *const T,
    len: usize,
    index: usize,
}

impl<T> UnsafePtrIter<T> {
    pub fn new(data: *const T, len: usize) -> Self {
        Self {
            data,
            len,
            index: 0,
        }
    }
}

impl<T: 'static> Iterator for UnsafePtrIter<T> {
    type Item = &'static T;

    /// SAFETY: Remember to ensure that every element of ptr + len is valid!
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }

        let val = unsafe { self.data.add(self.index).as_ref_unchecked() };
        self.index += 1;
        Some(val)
    }
}
