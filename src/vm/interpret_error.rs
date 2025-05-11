use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum InterpretError {
    CompileTime(String),
    Runtime(String),
}

impl Display for InterpretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompileTime(msg) => {
                f.write_fmt(format_args!("Encountered compile-time error.\n{}", msg))
            }
            Self::Runtime(msg) => f.write_fmt(format_args!("Encountered runtime error.\n{}", msg)),
        }
    }
}

impl Error for InterpretError {}

pub type InterpretResult = Result<(), InterpretError>;
