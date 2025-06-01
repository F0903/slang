use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Neg, Sub},
};

use super::{
    object::{Object, ObjectNode},
    value_casts::ValueCasts,
    value_type::ValueType,
};
use crate::{error::Error, memory::HeapPtr};

#[derive(Clone)]
pub struct Value {
    value_type: ValueType,
    casts: ValueCasts,
}

impl Value {
    pub const fn object(object_node: HeapPtr<ObjectNode>) -> Self {
        Self {
            value_type: ValueType::Object,
            casts: ValueCasts { object_node },
        }
    }

    pub const fn boolean(value: bool) -> Self {
        Self {
            value_type: ValueType::Bool,
            casts: ValueCasts { boolean: value },
        }
    }

    pub const fn number(value: f64) -> Self {
        Self {
            value_type: ValueType::Number,
            casts: ValueCasts { number: value },
        }
    }

    pub const fn none() -> Self {
        Self {
            value_type: ValueType::None,
            casts: ValueCasts { boolean: false },
        }
    }

    pub const fn as_number(&self) -> f64 {
        unsafe { self.casts.number }
    }

    pub const fn as_boolean(&self) -> bool {
        unsafe { self.casts.boolean }
    }

    pub const fn as_object_ptr(&self) -> HeapPtr<ObjectNode> {
        unsafe { self.casts.object_node }
    }

    pub fn is_falsey(&self) -> bool {
        self.value_type == ValueType::None
            || (self.value_type == ValueType::Bool && unsafe { !self.casts.boolean })
    }

    pub fn is_object(&self) -> bool {
        self.value_type == ValueType::Object
    }

    pub const fn get_type(&self) -> ValueType {
        self.value_type
    }
}

impl Add for Value {
    type Output = Result<Value, Error>;

    fn add(self, rhs: Self) -> Self::Output {
        match self.value_type {
            ValueType::Number => Ok(Value::number(self.as_number() + rhs.as_number())),
            ValueType::Bool => Err(Error::Runtime("Cannot add boolean values!".to_owned())),
            ValueType::Object => Err(Error::Runtime("Cannot add Object types!".to_owned())),
            ValueType::None => Err(Error::Runtime("Cannot add None types!".to_owned())),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, Error>;

    fn sub(self, rhs: Self) -> Self::Output {
        match self.value_type {
            ValueType::Number => Ok(Value::number(self.as_number() - rhs.as_number())),
            ValueType::Bool => Err(Error::Runtime("Cannot subtract boolean values!".to_owned())),
            ValueType::Object => Err(Error::Runtime("Cannot subtract Object types!".to_owned())),
            ValueType::None => Err(Error::Runtime("Cannot subtract None types!".to_owned())),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, Error>;

    fn mul(self, rhs: Self) -> Self::Output {
        match self.value_type {
            ValueType::Number => Ok(Value::number(self.as_number() * rhs.as_number())),
            ValueType::Bool => Err(Error::Runtime("Cannot multiply boolean values!".to_owned())),
            ValueType::Object => Err(Error::Runtime("Cannot multiply Object types!".to_owned())),
            ValueType::None => Err(Error::Runtime("Cannot multiply None types!".to_owned())),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, Error>;

    fn div(self, rhs: Self) -> Self::Output {
        match self.value_type {
            ValueType::Number => Ok(Value::number(self.as_number() / rhs.as_number())),
            ValueType::Bool => Err(Error::Runtime("Cannot divide boolean values!".to_owned())),
            ValueType::Object => Err(Error::Runtime("Cannot divide Object types!".to_owned())),
            ValueType::None => Err(Error::Runtime("Cannot divide None types!".to_owned())),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, Error>;

    fn neg(self) -> Self::Output {
        match self.value_type {
            ValueType::Number => Ok(Value::number(-self.as_number())),
            ValueType::Bool => Err(Error::Runtime("Cannot negate boolean values!".to_owned())),
            ValueType::Object => Err(Error::Runtime("Cannot negate Object types!".to_owned())),
            ValueType::None => Err(Error::Runtime("Cannot negate None types!".to_owned())),
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
                ValueType::Object => *self.casts.object_node == *other.casts.object_node,
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
                ValueType::Object => *self.casts.object_node > *other.casts.object_node,
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
                ValueType::Object => *self.casts.object_node >= *other.casts.object_node,
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
                ValueType::Object => *self.casts.object_node < *other.casts.object_node,
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
                ValueType::Object => *self.casts.object_node <= *other.casts.object_node,
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
                let obj = self.casts.object_node.get_object();
                match obj {
                    Object::String(s) => f.write_fmt(format_args!("String object: {}", s.as_str())),
                    Object::Function(func) => {
                        f.write_fmt(format_args!("Function object: {:?}", func.name))
                    }
                    Object::NativeFunction(func) => {
                        f.write_fmt(format_args!("NativeFunction object: {:?}", func))
                    }
                }
            },
            ValueType::None => f.write_str("None"),
        }
    }
}

impl Debug for Value {
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
                let obj = self.casts.object_node.get_object();
                match obj {
                    Object::String(s) => {
                        f.write_fmt(format_args!("String object: {:?}", s.as_str()))
                    }
                    Object::Function(func) => {
                        f.write_fmt(format_args!("Function object: {:?}", func.name))
                    }
                    Object::NativeFunction(func) => {
                        f.write_fmt(format_args!("NativeFunction object: {:?}", func))
                    }
                }
            },
            ValueType::None => f.write_str("None"),
        }
    }
}
