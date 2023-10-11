use super::{callable::CallableClone, Callable};
use crate::{
    environment::{EnvPtr, Environment},
    error::RuntimeError,
    interpreter::{Interpreter, MaybeReturn},
    statement::FunctionStatement,
    value::Value,
};
use std::{error::Error, fmt::Display};

#[derive(Clone, Copy)]
pub enum FunctionKind {
    Function,
    Method,
}

pub enum RuntimeOrNativeError {
    Runtime(RuntimeError),
    Native(Box<dyn std::error::Error>),
}

pub type NativeFunctionResult = Result<Value, Box<dyn Error>>;
pub type FunctionResult = Result<Value, RuntimeOrNativeError>;

impl Display for FunctionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Function => "Function",
            Self::Method => "Method",
        };
        f.write_str(text)
    }
}

#[derive(Debug, Clone)]
pub struct NativeFunction {
    name: String,
    arg_count: usize,
    func: fn(env: EnvPtr, args: Vec<Value>) -> NativeFunctionResult,
}

impl NativeFunction {
    pub fn new(
        name: impl ToString,
        arg_count: usize,
        func: fn(env: EnvPtr, args: Vec<Value>) -> NativeFunctionResult,
    ) -> Self {
        Self {
            name: name.to_string(),
            arg_count,
            func,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    declaration: FunctionStatement,
    closure: Environment,
}

impl Function {
    pub fn new(declaration: FunctionStatement, closure: Environment) -> Self {
        Self {
            declaration,
            closure,
        }
    }
}

impl CallableClone for Function {
    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }
}

impl Callable for Function {
    fn call(&mut self, interpreter: &mut Interpreter, args: Vec<Value>) -> FunctionResult {
        let mut local_env = Environment::new(Some(self.closure.clone().into()));
        for (param, arg) in self.declaration.params.iter().zip(args.iter()) {
            local_env.define(param.lexeme.clone(), arg.clone());
        }
        if let MaybeReturn::Return(x) = interpreter
            .execute_block(&self.declaration.body, local_env.into())
            .map_err(|x| RuntimeOrNativeError::Runtime(x))?
        {
            Ok(x)
        } else {
            Ok(Value::None)
        }
    }

    fn get_arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn get_name(&self) -> &str {
        &self.declaration.name.lexeme
    }
}

impl CallableClone for NativeFunction {
    fn clone_box(&self) -> Box<dyn Callable> {
        Box::new(self.clone())
    }
}

impl Callable for NativeFunction {
    fn call(&mut self, interpreter: &mut Interpreter, args: Vec<Value>) -> FunctionResult {
        let func_result = (self.func)(interpreter.get_current_env(), args);
        func_result.map_err(|e| RuntimeOrNativeError::Native(e))
    }

    fn get_arity(&self) -> usize {
        self.arg_count
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}
