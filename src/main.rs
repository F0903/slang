#![feature(str_from_raw_parts)]
#![feature(ptr_as_ref_unchecked)]
#![feature(specialization)]
#![feature(maybe_uninit_slice)]

mod collections;
mod compiler;
mod debug;
mod encoding;
mod error;
mod hashing;
mod lexing;
mod memory;
mod utils;
mod value;
mod vm;

use std::{
    env::args,
    io::{BufRead, Read, Write},
};

use vm::Vm;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn print_usage() {
    println!("Usage: slang for REPL or slang <path> to run a file.");
}

fn repl(vm: &mut Vm) -> Result<()> {
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

fn run_file(path: String, vm: &mut Vm) -> Result<()> {
    let mut buf = vec![];
    std::fs::File::open(path)?.read_to_end(&mut buf)?;
    interpret(&buf, vm)
}

fn interpret(buf: &[u8], vm: &mut Vm) -> Result<()> {
    //let mut vm = Vm::new(); // Create a new VM each time to debug the drop implementations.
    vm.interpret(buf)?;
    Ok(())
}

fn main() -> Result<()> {
    let mut args = args();
    let mut vm = Vm::new();
    match args.len() {
        1 => repl(&mut vm),
        2 => run_file(args.nth(1).unwrap(), &mut vm),
        _ => {
            print_usage();
            return Err("ERROR: Invalid usage.".into());
        }
    }
}
