use crate::types::{Argument, ScriptFunction, Value};

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub name: String,
    func: fn(Vec<Argument>) -> Value, // Args = Input values
}

impl NativeFunction {
    pub fn new(name: impl ToString, func: fn(Vec<Argument>) -> Value) -> Self {
        Self {
            name: name.to_string(),
            func,
        }
    }

    pub fn call(&self, args: Vec<Argument>) -> Value {
        (self.func)(args)
    }
}

#[derive(Debug, Clone)]
pub enum Function {
    Native(NativeFunction),
    Script(ScriptFunction),
}

impl Function {
    pub fn get_name(&self) -> &str {
        match self {
            Self::Native(x) => &x.name,
            Self::Script(x) => &x.name,
        }
    }
}
