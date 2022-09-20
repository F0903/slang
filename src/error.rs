use crate::token::Token;
use once_cell::sync::Lazy;
use std::{
    error::Error,
    fmt::Display,
    io::{stderr, Write},
    sync::{Mutex, MutexGuard},
};

pub type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug)]
pub struct RuntimeError {
    token: Token,
    msg: String,
}

impl RuntimeError {
    pub fn new(token: Token, msg: impl ToString) -> Self {
        Self {
            token,
            msg: msg.to_string(),
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl Error for RuntimeError {}

impl<S: ToString> From<(Token, S)> for RuntimeError {
    fn from(x: (Token, S)) -> Self {
        Self {
            token: x.0,
            msg: x.1.to_string(),
        }
    }
}

static ERR_HANDLER: Lazy<Mutex<Box<(dyn ErrorHandler + Sync + Send)>>> = Lazy::new(|| {
    Mutex::new(Box::new(StdErrorHandler {
        had_error: false,
        had_runtime_error: false,
    }))
});

pub fn get_err_handler<'a>() -> MutexGuard<'a, Box<(dyn ErrorHandler + Send + Sync)>> {
    ERR_HANDLER.lock().unwrap()
}

pub trait ErrorHandler {
    fn had_error(&self) -> bool;
    fn report(&self, line: usize, msg: &str);
    fn error(&mut self, token: Token, msg: &str);
    fn runtime_error(&mut self, err: RuntimeError);
}

pub struct StdErrorHandler {
    had_error: bool,
    had_runtime_error: bool,
}

impl ErrorHandler for StdErrorHandler {
    fn had_error(&self) -> bool {
        self.had_error
    }

    fn report(&self, line: usize, msg: &str) {
        stderr()
            .write_fmt(format_args!("{msg} at line {line}\n"))
            .ok();
    }

    fn error(&mut self, token: Token, msg: &str) {
        self.report(token.line, msg);
        self.had_error = true;
    }

    fn runtime_error(&mut self, err: RuntimeError) {
        self.report(err.token.line, &err.msg);
        self.had_runtime_error = true;
    }
}
