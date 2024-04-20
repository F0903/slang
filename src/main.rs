use std::{
    env::args,
    io::{BufRead, Read, Write},
};

use vm::VM;

mod chunk;
mod compiler;
mod debug;
mod dynarray;
mod encoding;
mod light_stack;
mod memory;
mod opcode;
mod parser;
mod scanner;
mod token;
mod utils;
mod value;
mod vm;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn print_usage() {
    println!("Usage: slang for REPL or slang <path> to run a file.");
}

fn repl(vm: &mut VM) -> Result<()> {
    let mut input = std::io::stdin().lock();
    let mut line_buf = String::new();
    loop {
        fprint!("> ");
        let read = input.read_line(&mut line_buf)?;
        if read < 1 {
            println!();
            break;
        }
        interpret(vm, line_buf.as_bytes())?;
        line_buf.clear();
        println!();
    }
    Ok(())
}

fn run_file(vm: &mut VM, path: String) -> Result<()> {
    let mut buf = vec![];
    std::fs::File::open(path)?.read_to_end(&mut buf)?;
    interpret(vm, &buf)
}

fn interpret(vm: &mut VM, buf: &[u8]) -> Result<()> {
    vm.interpret(buf)?;
    Ok(())
}

fn main() -> Result<()> {
    let mut args = args();
    let mut vm = VM::new();

    match args.len() {
        1 => repl(&mut vm),
        2 => run_file(&mut vm, args.nth(1).unwrap()),
        _ => {
            print_usage();
            return Err("ERROR: Invalid usage.".into());
        }
    }
}
