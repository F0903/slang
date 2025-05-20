use super::Stack;

// An iterator that iterates over a stack from the top to the bottom.
pub struct StackIter<'a, T, const STACK_SIZE: usize = 1024> {
    stack: &'a Stack<T, STACK_SIZE>,
    index: usize,
}

impl<'a, T, const STACK_SIZE: usize> StackIter<'a, T, STACK_SIZE> {
    pub const fn new(stack: &'a Stack<T, STACK_SIZE>) -> Self {
        Self {
            stack,
            index: stack.count(),
        }
    }
}

impl<'a, T, const STACK_SIZE: usize> Iterator for StackIter<'a, T, STACK_SIZE> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.stack.peek(self.index);
        self.index -= 1;
        Some(item)
    }
}
