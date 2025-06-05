use std::fmt::{Debug, Display};

use crate::{
    collections::DynArray,
    memory::HeapPtr,
    value::object::{self, Function},
};

#[derive(Clone)]
pub struct Closure {
    // The closure doesn't own the function so we don't dealloc it
    pub function: Function,
    upvalues: DynArray<HeapPtr<object::Upvalue>>,
}

impl Closure {
    pub fn new(function: Function, upvalues: DynArray<HeapPtr<object::Upvalue>>) -> Self {
        Self { function, upvalues }
    }

    pub const fn get_upvalues_count(&self) -> u16 {
        // This is guaranteed to never be over 255 (as currently defined)
        self.upvalues.get_count() as u16
    }

    pub fn get_upvalue(&self, index: usize) -> HeapPtr<object::Upvalue> {
        self.upvalues.copy_read(index as usize)
    }
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        self.function == other.function
    }
}

impl Display for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("closure (")?;
        Display::fmt(&self.function, f)?;
        f.write_str(")")
    }
}

impl Debug for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("closure (")?;
        Display::fmt(&self.function, f)?;
        f.write_str(")")
    }
}
