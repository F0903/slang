use std::{fmt::Debug, mem::MaybeUninit};

use crate::dbg_println;

pub struct Stack<T, const STACK_SIZE: usize = 1024> {
    stack: [MaybeUninit<T>; STACK_SIZE],
    count: usize,
}

impl<'a, T, const STACK_SIZE: usize> Stack<T, STACK_SIZE>
where
    T: Debug,
{
    pub const fn new() -> Self {
        Self {
            stack: [const { MaybeUninit::uninit() }; STACK_SIZE],
            count: 0,
        }
    }

    pub const fn count(&self) -> usize {
        self.count
    }

    pub const fn push(&mut self, val: T) {
        self.stack[self.count].write(val);
        self.count += 1;
    }

    const fn get_top(&self, offset_from_top: usize) -> &MaybeUninit<T> {
        &self.stack[self.count - 1 - offset_from_top]
    }

    const fn get_top_mut(&mut self, offset_from_top: usize) -> &mut MaybeUninit<T> {
        &mut self.stack[self.count - 1 - offset_from_top]
    }

    pub const fn get_top_mut_ref(&mut self) -> &mut T {
        unsafe { self.get_top_mut(0).assume_init_mut() }
    }

    pub const fn pop(&mut self) -> T {
        unsafe {
            let val = self.get_top(0).assume_init_read();
            self.count -= 1;
            val
        }
    }

    pub const fn peek(&self, distance: usize) -> &T {
        unsafe { self.get_top(distance).assume_init_ref() }
    }
}

impl<T, const STACK_SIZE: usize> Drop for Stack<T, STACK_SIZE> {
    fn drop(&mut self) {
        for i in 0..self.count {
            unsafe { self.stack[i].assume_init_drop() };
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
