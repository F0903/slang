mod value_casts;
mod value_type;

use value_casts::*;
use value_type::*;

use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Neg, Sub},
};

use crate::vm::InterpretError;

#[derive(Debug, Clone)]
pub struct Value {
    value_type: ValueType,
    casts: ValueCasts,
}

impl Value {
    pub fn boolean(value: bool) -> Self {
        Self {
            value_type: ValueType::Bool,
            casts: ValueCasts { boolean: value },
        }
    }

    pub fn number(value: f64) -> Self {
        Self {
            value_type: ValueType::Number,
            casts: ValueCasts { number: value },
        }
    }

    pub fn none() -> Self {
        Self {
            value_type: ValueType::None,
            casts: ValueCasts { boolean: false },
        }
    }

    pub fn as_number(&self) -> f64 {
        unsafe { self.casts.number }
    }

    pub fn as_boolean(&self) -> bool {
        unsafe { self.casts.boolean }
    }
}

impl Add for Value {
    type Output = Result<Value, InterpretError>;

    fn add(self, rhs: Self) -> Self::Output {
        match self.value_type {
            ValueType::Number => Ok(Value::number(self.as_number() + rhs.as_number())),
            ValueType::Bool => Err(InterpretError::Runtime(
                "Cannot add boolean values!".to_owned(),
            )),
            ValueType::None => Err(InterpretError::Runtime("Cannot add None types!".to_owned())),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, InterpretError>;

    fn sub(self, rhs: Self) -> Self::Output {
        match self.value_type {
            ValueType::Number => Ok(Value::number(self.as_number() - rhs.as_number())),
            ValueType::Bool => Err(InterpretError::Runtime(
                "Cannot subtract boolean values!".to_owned(),
            )),
            ValueType::None => Err(InterpretError::Runtime(
                "Cannot subtract None types!".to_owned(),
            )),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, InterpretError>;

    fn mul(self, rhs: Self) -> Self::Output {
        match self.value_type {
            ValueType::Number => Ok(Value::number(self.as_number() * rhs.as_number())),
            ValueType::Bool => Err(InterpretError::Runtime(
                "Cannot multiply boolean values!".to_owned(),
            )),
            ValueType::None => Err(InterpretError::Runtime(
                "Cannot multiply None types!".to_owned(),
            )),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, InterpretError>;

    fn div(self, rhs: Self) -> Self::Output {
        match self.value_type {
            ValueType::Number => Ok(Value::number(self.as_number() / rhs.as_number())),
            ValueType::Bool => Err(InterpretError::Runtime(
                "Cannot divide boolean values!".to_owned(),
            )),
            ValueType::None => Err(InterpretError::Runtime(
                "Cannot divide None types!".to_owned(),
            )),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, InterpretError>;

    fn neg(self) -> Self::Output {
        match self.value_type {
            ValueType::Number => Ok(Value::number(-self.as_number())),
            ValueType::Bool => Err(InterpretError::Runtime(
                "Cannot negate boolean values!".to_owned(),
            )),
            ValueType::None => Err(InterpretError::Runtime(
                "Cannot negate None types!".to_owned(),
            )),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value_type {
            ValueType::Bool => f.write_fmt(format_args!(
                "[{}] = {}",
                self.value_type,
                self.as_boolean()
            )),
            ValueType::Number => {
                f.write_fmt(format_args!("[{}] = {}", self.value_type, self.as_number()))
            }
            ValueType::None => f.write_str("None"),
        }
    }
}
