use crate::{error::Result, value::Value};

#[derive(Debug, Clone, PartialEq)]
pub struct NativeFunction {
    pub function: fn(u8, &[Value]) -> Result<Value>,
}

impl NativeFunction {
    pub fn new(function: fn(u8, &[Value]) -> Result<Value>) -> Self {
        Self { function }
    }

    pub fn call(&self, arity: u8, args: &[Value]) -> Result<Value> {
        (self.function)(arity, args)
    }
}
