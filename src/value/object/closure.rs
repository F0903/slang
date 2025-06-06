use std::fmt::Display;

use crate::{
    collections::DynArray,
    value::object::{self, Function, ObjectRef},
};

pub struct Closure {
    pub function: ObjectRef<Function>,
    upvalues: DynArray<ObjectRef<object::Upvalue>>,
}

impl Closure {
    pub fn new(
        function: ObjectRef<Function>,
        upvalues: DynArray<ObjectRef<object::Upvalue>>,
    ) -> Self {
        Self { function, upvalues }
    }

    pub const fn get_upvalues_count(&self) -> u16 {
        // This is guaranteed to never be over 255 (as currently defined)
        self.upvalues.get_count() as u16
    }

    pub fn get_upvalue(&self, index: usize) -> ObjectRef<object::Upvalue> {
        self.upvalues.copy_read(index as usize)
    }
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        *self.function == *other.function
    }
}

impl Display for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("closure (")?;
        Display::fmt(self.function.as_ref(), f)?;
        f.write_str(")")
    }
}
