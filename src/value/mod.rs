use std::fmt::{Debug, Display};

mod callable;
mod class;
mod function;

pub use {callable::Callable, class::Class, function::*};

#[derive(Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Callable(Box<dyn Callable>),
    Class(Class),
    None,
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Self::String(x) => Self::String(x.clone()),
            Self::Number(x) => Self::Number(x.clone()),
            Self::Boolean(x) => Self::Boolean(x.clone()),
            Self::Callable(x) => Self::Callable(x.clone_box()),
            Self::Class(x) => Self::Class(x.clone()),
            Self::None => Self::None,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(x) => f.write_fmt(format_args!("{x}")),
            Value::Number(x) => f.write_fmt(format_args!("{x}")),
            Value::Boolean(x) => f.write_fmt(format_args!("{x}")),
            Value::Callable(_) => f.write_str("<function>"),
            Value::Class(x) => Debug::fmt(x, f),
            Value::None => f.write_str("none"),
        }
    }
}
