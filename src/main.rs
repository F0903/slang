mod core_lib;
mod expressions;
mod keyword;
mod line_reader;
mod operators;
mod parser;
mod types;
mod util;
mod vm;

use types::{Argument, Value};
use vm::{NativeFunction, VirtualMachine};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(debug_assertions)]
const DEBUG_FILE: &str = include_str!("../test.cah");

#[cfg(debug_assertions)]
fn test(_args: Vec<Argument>) -> Value {
    println!("test");
    Value::None
}

///DEBUG
#[cfg(debug_assertions)]
fn run() -> Result<()> {
    let source = DEBUG_FILE;

    let mut vm = VirtualMachine::new();
    core_lib::register_funcs(&mut vm);
    vm.register_native_func(NativeFunction::new("test", test));
    vm.execute_text(source)?;

    Ok(())
}

///RELEASE
#[cfg(not(debug_assertions))]
fn run() -> Result<()> {
    let input_arg = std::env::args()
        .nth(1)
        .ok_or("Could not get input argument. Please specify the file to interpret.")?;

    let parser = Parser::new();
    let mut vm = VirtualMachine::new(parser);
    vm.execute_file(&input_arg)?;

    Ok(())
}

fn main() -> Result<()> {
    run()
}
