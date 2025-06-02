use crate::{error::Result, value::Value};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeFunction {
    pub function: fn(&[Value]) -> Result<Value>,
    pub arity: u8,
    pub name: &'static str,
}

impl NativeFunction {
    pub const fn new(
        function: fn(&[Value]) -> Result<Value>,
        arity: u8,
        name: &'static str,
    ) -> Self {
        Self {
            function,
            arity,
            name,
        }
    }
}

unsafe impl Send for NativeFunction {}
unsafe impl Sync for NativeFunction {}
