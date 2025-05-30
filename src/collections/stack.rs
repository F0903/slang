use std::{fmt::Debug, mem::MaybeUninit};

use super::{stack_iter::StackIter, stack_offset::StackOffset};

pub const DEFAULT_STACK_SIZE: usize = 1024;

pub struct Stack<T, const STACK_SIZE: usize = DEFAULT_STACK_SIZE> {
    stack: [MaybeUninit<T>; STACK_SIZE],
    count: usize,
}

impl<T, const STACK_SIZE: usize> Stack<T, STACK_SIZE> {
    pub const fn new() -> Self {
        debug_assert!(STACK_SIZE > 0, "stack size must not be 0");
        Self {
            stack: [const { MaybeUninit::uninit() }; STACK_SIZE],
            count: 0,
        }
    }

    pub const fn stack_size(&self) -> usize {
        STACK_SIZE
    }

    pub const fn count(&self) -> usize {
        self.count
    }

    pub const fn push(&mut self, val: T) {
        debug_assert!(self.count() < STACK_SIZE, "stack overflow");
        self.stack[self.count()].write(val);
        self.count += 1;
    }

    pub fn set_at(&mut self, index: usize, value: T) {
        debug_assert!(index < STACK_SIZE, "index is out of bounds");

        if index < self.count() {
            unsafe {
                self.stack[index].assume_init_drop(); // Drop the old value at index
            }
        }

        self.stack[index] = MaybeUninit::new(value);
    }

    pub fn pop(&mut self) -> T {
        debug_assert!(self.count > 0, "cannot pop from an empty stack");
        let maybe_init = &mut self.stack[self.count() - 1];
        let val = unsafe { maybe_init.assume_init_read() }; // First duplicate the value
        self.count -= 1;
        val
    }

    pub fn pop_n(&mut self, n: usize) -> &mut [T] {
        debug_assert!(self.count >= n, "cannot pop more than available elements");
        let count = self.count();
        let maybe_init = &mut self.stack[count - n..];
        let val = unsafe { maybe_init.assume_init_mut() }; // First duplicate the value
        self.count -= n;
        val
    }

    pub const fn get_at(&self, index: usize) -> T {
        debug_assert!(index < self.count(), "index is out of bounds");
        unsafe { self.stack[index].assume_init_read() }
    }

    pub const fn get_ref_at(&self, index: usize) -> &T {
        debug_assert!(index < self.count(), "index is out of bounds");
        unsafe { self.stack[index].assume_init_ref() }
    }

    pub const fn get_mut_at(&mut self, index: usize) -> &mut T {
        debug_assert!(index < self.count(), "index is out of bounds");
        unsafe { self.stack[index].assume_init_mut() }
    }

    pub fn top_ref(&self) -> &T {
        self.get_ref_at(self.count() - 1)
    }

    pub fn top_mut(&mut self) -> &mut T {
        self.get_mut_at(self.count() - 1)
    }

    pub const fn top_offset(&self, offset_from_top: usize) -> T {
        self.get_at(self.count() - 1 - offset_from_top)
    }

    pub const fn top_mut_offset(&mut self, offset_from_top: usize) -> &mut T {
        self.get_mut_at(self.count() - 1 - offset_from_top)
    }

    pub const fn peek(&self, offset_from_top: usize) -> &T {
        self.get_ref_at(self.count() - 1 - offset_from_top)
    }

    pub const fn iter<'a>(&'a self) -> StackIter<'a, T, STACK_SIZE> {
        StackIter::new(self)
    }

    pub fn offset<'a>(&'a mut self, offset_from_base: usize) -> StackOffset<'a, T, STACK_SIZE> {
        StackOffset::new(self, offset_from_base)
    }
}

impl<T, const STACK_SIZE: usize> Drop for Stack<T, STACK_SIZE> {
    fn drop(&mut self) {
        unsafe {
            let ceil = self.count();
            self.stack[..ceil].assume_init_drop();
        }
        self.count = 0;
    }
}

impl<T, const STACK_SIZE: usize> Debug for Stack<T, STACK_SIZE>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("          ")?;
        for i in 0..self.count {
            let elem = unsafe { self.stack[i].assume_init_ref() };
            f.write_str("[ ")?;
            f.write_fmt(format_args!("{:?}", elem))?;
            f.write_str(" ]")?;
        }
        f.write_str("\n")?;
        Ok(())
    }
}
