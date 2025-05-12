#[derive(Debug)]
pub struct BorrowedIter<'a, T> {
    data: &'a [T],
    index: usize,
}

impl<'a, T> BorrowedIter<'a, T> {
    pub const fn new(data: &'a [T]) -> Self {
        Self { data, index: 0 }
    }
}

impl<'a, T> Iterator for BorrowedIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let item = &self.data[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
