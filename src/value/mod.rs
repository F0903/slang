pub mod object;
mod value_casts;
mod value_type;

use object::{Object, ObjectContainer};
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
    pub fn object(value: ObjectContainer) -> Self {
        Self {
            value_type: ValueType::Object,
            casts: ValueCasts { object: value },
        }
    }

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

    pub fn as_object_ptr(&self) -> ObjectContainer {
        unsafe { self.casts.object }
    }

    pub fn is_falsey(&self) -> bool {
        self.value_type == ValueType::None
            || (self.value_type == ValueType::Bool && unsafe { !self.casts.boolean })
    }

    pub fn is_object(&self) -> bool {
        self.value_type == ValueType::Object
    }

    pub fn get_type(&self) -> ValueType {
        self.value_type
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
            ValueType::Object => Err(InterpretError::Runtime(
                "Cannot add Object types!".to_owned(),
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
            ValueType::Object => Err(InterpretError::Runtime(
                "Cannot subtract Object types!".to_owned(),
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
            ValueType::Object => Err(InterpretError::Runtime(
                "Cannot multiply Object types!".to_owned(),
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
            ValueType::Object => Err(InterpretError::Runtime(
                "Cannot divide Object types!".to_owned(),
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
            ValueType::Object => Err(InterpretError::Runtime(
                "Cannot negate Object types!".to_owned(),
            )),
            ValueType::None => Err(InterpretError::Runtime(
                "Cannot negate None types!".to_owned(),
            )),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if self.value_type != other.value_type {
            return false;
        }
        unsafe {
            match self.value_type {
                ValueType::Bool => self.casts.boolean == other.casts.boolean,
                ValueType::Number => self.casts.number == other.casts.number,
                ValueType::Object => self.casts.object == other.casts.object,
                ValueType::None => other.value_type == ValueType::None,
            }
        }
    }
}

impl PartialOrd for Value {
    fn gt(&self, other: &Self) -> bool {
        if self.value_type != other.value_type {
            return false;
        }
        unsafe {
            match self.value_type {
                ValueType::Bool => self.casts.boolean && !other.casts.boolean,
                ValueType::Number => self.casts.number > other.casts.number,
                ValueType::Object => self.casts.object > other.casts.object,
                ValueType::None => false,
            }
        }
    }

    fn ge(&self, other: &Self) -> bool {
        if self.value_type != other.value_type {
            return false;
        }
        unsafe {
            match self.value_type {
                ValueType::Bool => {
                    (self.casts.boolean && !other.casts.boolean)
                        || self.casts.boolean == other.casts.boolean
                }
                ValueType::Number => self.casts.number >= other.casts.number,
                ValueType::Object => self.casts.object >= other.casts.object,
                ValueType::None => false,
            }
        }
    }

    fn lt(&self, other: &Self) -> bool {
        if self.value_type != other.value_type {
            return false;
        }
        unsafe {
            match self.value_type {
                ValueType::Bool => !self.casts.boolean && other.casts.boolean,
                ValueType::Number => self.casts.number < other.casts.number,
                ValueType::Object => self.casts.object < other.casts.object,
                ValueType::None => false,
            }
        }
    }

    fn le(&self, other: &Self) -> bool {
        if self.value_type != other.value_type {
            return false;
        }
        unsafe {
            match self.value_type {
                ValueType::Bool => {
                    (!self.casts.boolean && other.casts.boolean)
                        || self.casts.boolean == other.casts.boolean
                }
                ValueType::Number => self.casts.number <= other.casts.number,
                ValueType::Object => self.casts.object <= other.casts.object,
                ValueType::None => false,
            }
        }
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.value_type != other.value_type {
            return None;
        } else if self > other {
            Some(std::cmp::Ordering::Greater)
        } else if self < other {
            Some(std::cmp::Ordering::Less)
        } else {
            Some(std::cmp::Ordering::Equal)
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
            ValueType::Object => unsafe {
                let obj_wrapper = self.casts.object;
                let obj_ptr = obj_wrapper.get_object();
                match &*obj_ptr {
                    Object::String(s) => {
                        f.write_fmt(format_args!("String object: {}", s.get_str()))
                    }
                }
            },
            ValueType::None => f.write_str("None"),
        }
    }
}
