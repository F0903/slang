use super::Stack;

// An iterator that iterates over a stack from the top to the bottom.
pub struct StackRevIter<'a, T, const STACK_SIZE: usize = 1024> {
    stack: &'a Stack<T, STACK_SIZE>,
    index: isize,
}

impl<'a, T, const STACK_SIZE: usize> StackRevIter<'a, T, STACK_SIZE> {
    pub const fn new(stack: &'a Stack<T, STACK_SIZE>) -> Self {
        Self { stack, index: 0 }
    }
}

impl<'a, T, const STACK_SIZE: usize> Iterator for StackRevIter<'a, T, STACK_SIZE> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.stack.count() as isize {
            return None;
        }

        let item = self.stack.get_ref_at(self.index as usize);
        self.index += 1;
        Some(item)
    }
}
