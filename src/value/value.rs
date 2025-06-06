use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Neg, Sub},
};

use super::object::Object;
use crate::{error::Error, memory::HeapPtr, value::object::InternedString};

#[derive(Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    String(InternedString),
    Object(HeapPtr<Object>),
    None,
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Bool(b) => !*b,
            Value::Number(n) => *n == 0.0,
            Value::String(s) => s.is_empty(),
            Value::Object(_) => false, // Objects are truthy
            Value::None => true,
        }
    }
}

impl Copy for Value {}

impl Add for Value {
    type Output = Result<Value, Error>;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Value::Number(num) => match rhs {
                Value::Number(rhs_num) => Ok(Value::Number(num + rhs_num)),
                _ => Err(Error::Runtime(
                    "Cannot add non-number types to numbers!".to_owned(),
                )),
            },
            Value::Bool(_) => Err(Error::Runtime("Cannot add boolean values!".to_owned())),
            Value::String(_) => unreachable!(),
            Value::Object(_) => Err(Error::Runtime("Cannot add Object types!".to_owned())),
            Value::None => Err(Error::Runtime("Cannot add None types!".to_owned())),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, Error>;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Value::Number(num) => match rhs {
                Value::Number(rhs_num) => Ok(Value::Number(num - rhs_num)),
                _ => Err(Error::Runtime(
                    "Cannot subtract non-number types to numbers!".to_owned(),
                )),
            },
            Value::Bool(_) => Err(Error::Runtime("Cannot subtract boolean values!".to_owned())),
            Value::String(_) => Err(Error::Runtime("Cannot subtract String types!".to_owned())),
            Value::Object(_) => Err(Error::Runtime("Cannot subtract Object types!".to_owned())),
            Value::None => Err(Error::Runtime("Cannot subtract None types!".to_owned())),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, Error>;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            Value::Number(num) => match rhs {
                Value::Number(rhs_num) => Ok(Value::Number(num * rhs_num)),
                _ => Err(Error::Runtime(
                    "Cannot multiply non-number types to numbers!".to_owned(),
                )),
            },
            Value::Bool(_) => Err(Error::Runtime("Cannot multiply boolean values!".to_owned())),
            Value::String(_) => Err(Error::Runtime("Cannot multiply String types!".to_owned())),
            Value::Object(_) => Err(Error::Runtime("Cannot multiply Object types!".to_owned())),
            Value::None => Err(Error::Runtime("Cannot multiply None types!".to_owned())),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, Error>;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            Value::Number(num) => match rhs {
                Value::Number(rhs_num) => Ok(Value::Number(num * rhs_num)),
                _ => Err(Error::Runtime(
                    "Cannot divide non-number types to numbers!".to_owned(),
                )),
            },
            Value::Bool(_) => Err(Error::Runtime("Cannot divide boolean values!".to_owned())),
            Value::String(_) => Err(Error::Runtime("Cannot divide String types!".to_owned())),
            Value::Object(_) => Err(Error::Runtime("Cannot divide Object types!".to_owned())),
            Value::None => Err(Error::Runtime("Cannot divide None types!".to_owned())),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, Error>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Number(num) => Ok(Value::Number(-num)),
            Value::Bool(_) => Err(Error::Runtime("Cannot negate boolean values!".to_owned())),
            Value::String(_) => Err(Error::Runtime("Cannot negate String types!".to_owned())),
            Value::Object(_) => Err(Error::Runtime("Cannot negate Object types!".to_owned())),
            Value::None => Err(Error::Runtime("Cannot negate None types!".to_owned())),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Bool(b) => match other {
                Value::Bool(other_b) => *b == *other_b,
                _ => false,
            },
            Value::Number(n) => match other {
                Value::Number(other_n) => *n == *other_n,
                _ => false,
            },
            Value::String(str) => match other {
                Value::String(other_str) => str == other_str,
                _ => false,
            },
            Value::Object(obj) => match other {
                Value::Object(other_obj) => *obj == *other_obj,
                _ => false,
            },
            Value::None => matches!(other, Value::None),
        }
    }
}

impl PartialOrd for Value {
    fn gt(&self, other: &Self) -> bool {
        match self {
            Value::Bool(b) => match other {
                Value::Bool(other_b) => return b > other_b,
                _ => return false,
            },
            Value::Number(n) => match other {
                Value::Number(other_n) => return n > other_n,
                _ => return false,
            },
            Value::String(_) => false,
            Value::Object(_) => false,
            Value::None => return false,
        }
    }

    fn ge(&self, other: &Self) -> bool {
        match self {
            Value::Bool(b) => match other {
                Value::Bool(other_b) => return b >= other_b,
                _ => return false,
            },
            Value::Number(n) => match other {
                Value::Number(other_n) => return n >= other_n,
                _ => return false,
            },
            Value::String(_) => false,
            Value::Object(_) => false,
            Value::None => return false,
        }
    }

    fn lt(&self, other: &Self) -> bool {
        match self {
            Value::Bool(b) => match other {
                Value::Bool(other_b) => return b < other_b,
                _ => return false,
            },
            Value::Number(n) => match other {
                Value::Number(other_n) => return n < other_n,
                _ => return false,
            },
            Value::String(_) => false,
            Value::Object(_) => false,
            Value::None => return false,
        }
    }

    fn le(&self, other: &Self) -> bool {
        match self {
            Value::Bool(b) => match other {
                Value::Bool(other_b) => return b <= other_b,
                _ => return false,
            },
            Value::Number(n) => match other {
                Value::Number(other_n) => return n <= other_n,
                _ => return false,
            },
            Value::String(_) => false,
            Value::Object(_) => false,
            Value::None => return false,
        }
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self == other {
            return Some(std::cmp::Ordering::Equal);
        }
        if self > other {
            return Some(std::cmp::Ordering::Greater);
        }
        if self < other {
            return Some(std::cmp::Ordering::Less);
        }
        None
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => Display::fmt(b, f),
            Value::Number(num) => Display::fmt(num, f),
            Value::String(str) => Display::fmt(str, f),
            Value::Object(obj) => Display::fmt(obj, f),
            Value::None => f.write_str("None"),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => f.write_fmt(format_args!("[Bool] = {}", b)),
            Value::Number(num) => f.write_fmt(format_args!("[Number] = {}", num)),
            Value::String(str) => f.write_fmt(format_args!("[String] = \"{}\"", str)),
            Value::Object(obj) => Display::fmt(obj, f),
            Value::None => f.write_str("None"),
        }
    }
}
