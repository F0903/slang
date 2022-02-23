mod defs;
mod expression;
mod identifiable;
mod operators;
mod parser;
mod token;
mod util;
mod value;
mod vm;

use parser::Parser;
use vm::VirtualMachine;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const DEBUG_FILE: &str = include_str!("../test.cah");

fn main() -> Result<()> {
    /*
    std::env::args()
        .nth(1)
        .ok_or("Could not get input argument. Please specify the file to interpret.")?;
     */
    let source = DEBUG_FILE;
    let parser = Parser::new();
    let mut vm = VirtualMachine::new(parser);
    vm.execute_text(source)?;

    Ok(())
}
