use super::Stack;

// An iterator that iterates over a stack from the top to the bottom.
pub struct StackIter<'a, T, const STACK_SIZE: usize = 1024> {
    stack: &'a Stack<T, STACK_SIZE>,
    index: isize,
}

impl<'a, T, const STACK_SIZE: usize> StackIter<'a, T, STACK_SIZE> {
    pub const fn new(stack: &'a Stack<T, STACK_SIZE>) -> Self {
        Self {
            stack,
            index: stack.count() as isize - 1,
        }
    }
}

impl<'a, T, const STACK_SIZE: usize> Iterator for StackIter<'a, T, STACK_SIZE> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < 0 {
            return None;
        }

        let item = self.stack.get_ref_at(self.index as usize);
        self.index -= 1;
        Some(item)
    }
}
