use std::{fmt::Debug, ptr::null_mut};

use crate::memory::reallocate;

pub struct Stack<T, const STACK_SIZE: usize = 1024> {
    stack: *mut T,
    stack_top: *mut T,
}

impl<T, const STACK_SIZE: usize> Stack<T, STACK_SIZE> {
    pub fn new() -> Self {
        let stack = reallocate::<T>(null_mut(), 0, STACK_SIZE).cast();
        Self {
            stack,
            stack_top: stack,
        }
    }

    pub const fn count(&self) -> isize {
        unsafe { self.stack_top.offset_from(self.stack) }
    }

    pub fn get_top_mut_ref(&mut self) -> &mut T {
        unsafe { self.stack_top.sub(1).as_mut().unwrap() }
    }

    pub fn push(&mut self, val: T) {
        //TODO: consider making the stack resizable and check for cap/size here.
        unsafe {
            self.stack_top.write(val);
            self.stack_top = self.stack_top.add(1);
        }
    }

    pub fn pop(&mut self) -> T {
        //TODO: consider making the stack resizable and check for cap/size here.
        unsafe {
            self.stack_top = self.stack_top.sub(1);
            self.stack_top.read()
        }
    }

    pub fn peek(&self, distance: usize) -> &T {
        unsafe { &*self.stack_top.sub(distance + 1) }
    }
}

impl<T, const STACK_SIZE: usize> Drop for Stack<T, STACK_SIZE> {
    fn drop(&mut self) {
        reallocate::<T>(self.stack.cast(), STACK_SIZE, 0);
    }
}

impl<T, const STACK_SIZE: usize> Debug for Stack<T, STACK_SIZE>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.count();
        if count < 1 {
            return Ok(());
        }
        let mut ptr = self.stack;
        f.write_str("          ")?;
        loop {
            if ptr >= self.stack_top {
                break;
            }
            unsafe {
                f.write_str("[ ")?;
                let val = ptr.read();
                f.write_fmt(format_args!("{:?}", val))?;
                f.write_str(" ]")?;
                ptr = ptr.add(1);
            }
        }
        f.write_str("\n")?;
        Ok(())
    }
}
