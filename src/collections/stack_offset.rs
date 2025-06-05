use super::{Stack, stack::DEFAULT_STACK_SIZE};

pub struct StackOffset<'stack, T, const STACK_SIZE: usize = DEFAULT_STACK_SIZE> {
    stack: &'stack mut Stack<T, STACK_SIZE>,
    base_offset: usize,
}

impl<'stack, T, const STACK_SIZE: usize> StackOffset<'stack, T, STACK_SIZE> {
    pub fn new(stack: &'stack mut Stack<T, STACK_SIZE>, base_offset: usize) -> Self {
        Self { stack, base_offset }
    }

    pub fn set_at(&mut self, index: usize, value: T) {
        self.stack.set_at(self.base_offset + index, value)
    }

    pub fn get_at(&self, index: usize) -> T {
        self.stack.get_at(self.base_offset + index)
    }

    pub fn get_ref_at(&self, index: usize) -> &T {
        self.stack.get_ref_at(self.base_offset + index)
    }

    pub fn get_mut_at(&mut self, index: usize) -> &mut T {
        self.stack.get_mut_at(self.base_offset + index)
    }
}
