use super::{FunctionResult, Value};
use crate::interpreter::Interpreter;
use std::fmt::Debug;

pub trait CallableClone {
    fn clone_box(&self) -> Box<dyn Callable>;
}

pub trait Callable: CallableClone + Debug {
    fn call(&mut self, interpreter: &mut Interpreter, args: Vec<Value>) -> FunctionResult;
    fn get_arity(&self) -> usize;
    fn get_name(&self) -> &str;
}
