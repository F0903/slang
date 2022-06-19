use crate::{
    types::{Argument, Value},
    vm::{NativeFunction, VirtualMachine},
};

pub fn register_funcs(vm: &mut VirtualMachine) {
    vm.register_native_func(NativeFunction::new("print_line", print_line))
}

pub fn print_line(input: Vec<Argument>) -> Value {
    let input = &input[0];
    match &input.value {
        Value::Boolean(x) => println!("{}", x),
        Value::Number(x) => println!("{}", x),
        Value::String(x) => println!("{}", x),
        _ => println!(),
    }
    Value::None
}
