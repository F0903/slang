use std::fmt::Debug;

const STACK_MAX: usize = 1024;

pub struct LightStack<T> {
    stack: [T; STACK_MAX],
    stack_top: *mut T,
}

impl<T> LightStack<T>
where
    T: Default + Copy,
{
    /// CALL INIT BEFORE USING
    pub fn new() -> Self {
        let val = T::default();
        Self {
            stack: [val; STACK_MAX],
            stack_top: std::ptr::null_mut(), // Cannot create pointer to local value, will have to do in init()
        }
    }

    pub const fn count(&self) -> isize {
        unsafe { self.stack_top.offset_from(self.stack.as_ptr()) }
    }

    pub fn init(&mut self) {
        self.reset()
    }

    pub fn reset(&mut self) {
        self.stack_top = self.stack.as_mut_ptr();
    }

    pub fn get_top_mut_ref(&mut self) -> &mut T {
        unsafe { self.stack_top.sub(1).as_mut().unwrap() }
    }

    // UNCHECKED
    pub fn push(&mut self, val: T) {
        unsafe {
            self.stack_top.write(val);
            self.stack_top = self.stack_top.add(1);
        }
    }

    // UNCHECKED
    pub fn pop(&mut self) -> T {
        unsafe {
            self.stack_top = self.stack_top.sub(1);
            self.stack_top.read()
        }
    }
}

impl<T> Debug for LightStack<T>
where
    T: Default + Copy + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.count();
        if count < 1 {
            return Ok(());
        }
        let mut ptr = self.stack.as_ptr();
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
