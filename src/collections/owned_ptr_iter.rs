#[derive(Debug)]
pub struct OwnedPtrIter<T> {
    data: *mut T,
    count: usize,
    index: usize,
}

impl<T> OwnedPtrIter<T> {
    pub const fn new(data: *mut T, count: usize) -> Self {
        Self {
            data,
            count,
            index: 0,
        }
    }
}

impl<T> Iterator for OwnedPtrIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.count {
            let item = unsafe { self.data.add(self.index).read() };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
