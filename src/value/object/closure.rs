use std::fmt::Display;

use crate::{
    collections::DynArray,
    value::object::{self, Function},
};

#[derive(Debug, Clone)]
pub struct Closure {
    // The closure doesn't own the function so we don't dealloc it
    pub function: Function,
    upvalues: DynArray<object::Upvalue>,
}

impl Closure {
    pub fn new(function: Function, upvalues: DynArray<object::Upvalue>) -> Self {
        Self { function, upvalues }
    }

    pub const fn get_upvalues_count(&self) -> u16 {
        // This is guaranteed to never be over 255 (as currently defined)
        self.upvalues.get_count() as u16
    }

    pub fn get_upvalue_ref(&self, index: usize) -> &object::Upvalue {
        self.upvalues.get(index as usize)
    }

    pub fn get_upvalue_mut(&mut self, index: usize) -> &mut object::Upvalue {
        self.upvalues.get_mut(index as usize)
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
