use crate::{error::Result, value::Value};

#[derive(Debug, Clone, PartialEq)]
pub struct NativeFunction {
    pub function: fn(&[Value]) -> Result<Value>,
    pub arity: u8,
    pub name: String,
}

impl NativeFunction {
    pub fn new(function: fn(&[Value]) -> Result<Value>, arity: u8, name: String) -> Self {
        Self {
            function,
            arity,
            name,
        }
    }
}
