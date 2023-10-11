use super::{Callable, CallableResult, Instance, SharedPtr, Value};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
}

impl Class {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl<'a> Callable<'a> for Class {
    fn call(
        &mut self,
        _interpreter: &mut crate::interpreter::Interpreter,
        _args: Vec<super::Value>,
    ) -> CallableResult {
        //TODO: dont clone
        let instance = Instance::new(self.clone());
        CallableResult::Ok(Value::Instance(SharedPtr::new(instance)))
    }

    fn get_arity(&self) -> usize {
        return 0; //TODO custom constructors
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }
}
