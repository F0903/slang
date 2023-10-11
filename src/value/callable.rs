use super::{RuntimeOrNativeError, Value};
use crate::interpreter::Interpreter;
use std::fmt::Debug;

pub type CallableResult = Result<Value, RuntimeOrNativeError>;

pub trait Callable<'a>: Debug {
    fn call(&mut self, interpreter: &mut Interpreter, args: Vec<Value>) -> CallableResult;
    fn get_arity(&self) -> usize;
    fn get_name(&self) -> String;
}
