use std::{
    env::args,
    io::{BufRead, Read, Write},
};

use vm::GLOBAL_VM;

mod chunk;
mod compiler;
mod debug;
mod dynarray;
mod encoding;
mod memory;
mod opcode;
mod parser;
mod scanner;
mod stack;
mod token;
mod utils;
mod value;
mod vm;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn print_usage() {
    println!("Usage: slang for REPL or slang <path> to run a file.");
}

fn repl() -> Result<()> {
    let mut input = std::io::stdin().lock();
    let mut line_buf = String::new();
    loop {
        fprint!("> ");
        let read = input.read_line(&mut line_buf)?;
        if read < 1 {
            println!();
            break;
        }
        interpret(line_buf.as_bytes())?;
        line_buf.clear();
        println!();
    }
    Ok(())
}

fn run_file(path: String) -> Result<()> {
    let mut buf = vec![];
    std::fs::File::open(path)?.read_to_end(&mut buf)?;
    interpret(&buf)
}

fn interpret(buf: &[u8]) -> Result<()> {
    unsafe {
        GLOBAL_VM.interpret(buf)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let mut args = args();

    match args.len() {
        1 => repl(),
        2 => run_file(args.nth(1).unwrap()),
        _ => {
            print_usage();
            return Err("ERROR: Invalid usage.".into());
        }
    }
}
