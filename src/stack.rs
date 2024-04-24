use std::{fmt::Debug, mem::MaybeUninit};

pub struct Stack<T, const STACK_SIZE: usize = 1024> {
    stack: [MaybeUninit<T>; STACK_SIZE], // Be careful
    count: usize,
}

impl<'a, T, const STACK_SIZE: usize> Stack<T, STACK_SIZE> {
    const ARRAY_REPEAT_VALUE: MaybeUninit<T> = MaybeUninit::uninit();

    pub fn new() -> Self {
        Self {
            stack: [Self::ARRAY_REPEAT_VALUE; STACK_SIZE],
            count: 0,
        }
    }

    pub const fn count(&self) -> usize {
        self.count
    }

    pub fn push(&mut self, val: T) {
        self.stack[self.count].write(val);
        self.count += 1;
    }

    pub fn get_top_mut_ref(&mut self) -> &mut T {
        unsafe { self.stack[self.count - 1].assume_init_mut() }
    }

    pub fn pop(&mut self) -> T {
        unsafe {
            let val = self.stack[self.count - 1].assume_init_read();
            self.count -= 1;
            val
        }
    }

    pub const fn peek(&self, distance: usize) -> &T {
        unsafe { self.stack[self.count - distance].assume_init_ref() }
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
