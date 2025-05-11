#![feature(str_from_raw_parts)]
#![feature(layout_for_ptr)]
#![feature(get_mut_unchecked)]
#![feature(specialization)]

use std::{
    env::args,
    io::{BufRead, Read, Write},
};

use vm::VM;

mod chunk;
mod collections;
mod compiler;
mod debug;
mod encoding;
mod lexing;
mod memory;
mod opcode;
mod parser;
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
        interpret(line_buf.as_bytes(), vm)?;
        line_buf.clear();
        println!();
    }
    Ok(())
}

fn run_file(path: String, vm: &mut VM) -> Result<()> {
    let mut buf = vec![];
    std::fs::File::open(path)?.read_to_end(&mut buf)?;
    interpret(&buf, vm)
}

fn interpret(buf: &[u8], vm: &mut VM) -> Result<()> {
    vm.interpret(buf)?;
    Ok(())
}

fn main() -> Result<()> {
    let mut args = args();
    let mut vm = VM::new();
    match args.len() {
        1 => repl(&mut vm),
        2 => run_file(args.nth(1).unwrap(), &mut vm),
        _ => {
            print_usage();
            return Err("ERROR: Invalid usage.".into());
        }
    }
}
