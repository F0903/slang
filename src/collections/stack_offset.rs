use super::{Stack, stack::DEFAULT_STACK_SIZE};

pub struct StackOffset<'stack, T, const STACK_SIZE: usize = DEFAULT_STACK_SIZE> {
    stack: &'stack mut Stack<T, STACK_SIZE>,
    offset: usize,
}

impl<'stack, T, const STACK_SIZE: usize> StackOffset<'stack, T, STACK_SIZE> {
    pub fn new(stack: &'stack mut Stack<T, STACK_SIZE>, frame_offset: usize) -> Self {
        Self {
            stack,
            offset: frame_offset,
        }
    }

    pub fn peek(&self, offset_from_top: usize) -> &T {
        self.stack
            .get_ref_at((self.stack.count() - 1 - offset_from_top) + self.offset)
    }

    pub fn top_mut(&mut self, offset_from_top: usize) -> &mut T {
        self.stack
            .get_mut_at((self.stack.count() - 1 - offset_from_top) + self.offset)
    }

    pub fn set_at(&mut self, index: usize, value: T) {
        self.stack.set_at(index + self.offset, value)
    }

    pub fn get_at(&self, index: usize) -> T {
        self.stack.get_at(index + self.offset)
    }

    pub fn get_ref_at(&self, index: usize) -> &T {
        self.stack.get_ref_at(index + self.offset)
    }
}
