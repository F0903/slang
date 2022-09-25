mod environment;
mod error;
mod expression;
mod interpreter;
mod lexer;
mod parser;
mod statement;
mod token;
mod utils;
mod value;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use std::{
    env::args,
    fs::{canonicalize, File},
    io::{stdin, stdout, BufRead, Read, Write},
    path::Path,
};

use crate::value::{NativeFunction, Value};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn get_source_path() -> Option<String> {
    let mut args = args();
    args.nth(1)
}

fn run(source: String, interpreter: &mut Interpreter) -> Result<()> {
    interpreter.register_native(NativeFunction::new("hello_world".to_owned(), 0, |_, _| {
        println!("Hello world!");
        Ok(Value::None)
    }));

    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer);
    interpreter.interpret(parser);
    Ok(())
}

fn run_interactively() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let mut stdout = stdout().lock();
    let mut stdin = stdin().lock();
    let mut strbuf = String::new();
    loop {
        stdout.write_all(b"> ")?;
        stdout.flush()?;
        let count = stdin.read_line(&mut strbuf)?;
        if count == 0 {
            break;
        }
        run(strbuf.clone(), &mut interpreter).ok();
        strbuf.clear();
    }
    Ok(())
}

fn run_file(path: impl AsRef<Path>) -> Result<()> {
    let path = canonicalize(path)?;
    let mut interpreter = Interpreter::new();
    let mut buf = String::new();
    File::open(path)?.read_to_string(&mut buf)?;
    run(buf, &mut interpreter)?;
    Ok(())
}

fn main() -> Result<()> {
    match get_source_path() {
        Some(x) => run_file(x),
        None => run_interactively(),
    }
}
