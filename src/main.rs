mod defs;
mod expressions;
mod identifiable;
mod keyword;
mod line_reader;
mod operators;
mod parser;
mod value;
mod vm;

use vm::VirtualMachine;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(debug_assertions)]
const DEBUG_FILE: &str = include_str!("../test.cah");

///DEBUG
#[cfg(debug_assertions)]
fn run() -> Result<()> {
    let source = DEBUG_FILE;

    let mut vm = VirtualMachine::new();
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

///! Do not debug with any rust version later than 1.58.1 or breakpoints will not be hit.
fn main() -> Result<()> {
    run()
}
