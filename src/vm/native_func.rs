use crate::types::{Argument, Parameter, ScriptFunction, Value};

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub name: String,
    pub params: Vec<Parameter>,       // Params = Input names
    func: fn(Vec<Argument>) -> Value, // Args = Input values
}

impl NativeFunction {
    pub fn new(
        name: impl ToString,
        params: impl Into<Vec<Parameter>>,
        func: fn(Vec<Argument>) -> Value,
    ) -> Self {
        Self {
            name: name.to_string(),
            params: params.into(),
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

    pub fn get_params(&self) -> &Vec<Parameter> {
        match self {
            Self::Native(x) => &x.params,
            Self::Script(x) => &x.params,
        }
    }
}
