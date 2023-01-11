use crate::environment::EnvPtr;
use crate::interpreter::Interpreter;
use crate::value::{NativeFunction, NativeFunctionResult, Value};

pub fn register(interpreter: &mut Interpreter) {
    interpreter.register_native(NativeFunction::new("print", 1, print));
    interpreter.register_native(NativeFunction::new("print_line", 1, print_line));
    interpreter.register_native(NativeFunction::new("test_err", 0, test_err));
}

pub fn print(_env: EnvPtr, values: Vec<Value>) -> NativeFunctionResult {
    let val = values.first().unwrap();
    print!("{}", val);
    Ok(Value::None)
}

pub fn print_line(_env: EnvPtr, values: Vec<Value>) -> NativeFunctionResult {
    let val = values.first().unwrap();
    println!("{}", val);
    Ok(Value::None)
}

pub fn test_err(_env: EnvPtr, _values: Vec<Value>) -> NativeFunctionResult {
    Err("I'm here for testing purposes!".into())
}
