use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Neg, Sub},
};

use super::object::Object;
use crate::{error::Error, memory::HeapPtr, value::object::InternedString};

macro_rules! value_ctor {
    ($name:ident, $variant:ident, $ty:ty, $tag:expr) => {
        #[inline]
        pub const fn $name(val: $ty) -> Self {
            let inner = ValueUnion { $variant: val };
            Self::new($tag, inner)
        }
    };
}

macro_rules! value_as_fn {
    ($fn_name:ident, $variant:ident, $ty:ty, $tag:expr) => {
        #[inline]
        pub fn $fn_name(&self) -> $ty {
            debug_assert!(
                self.value_type == $tag,
                concat!(
                    "Tried to access a ",
                    stringify!($tag),
                    " value as a ",
                    stringify!($ty),
                    "!"
                )
            );
            unsafe { self.casts.$variant }
        }
    };
}

#[derive(Clone, Copy)]
union ValueUnion {
    bool: bool,
    number: f64,
    string: InternedString,
    object: HeapPtr<Object>,
}

#[derive(Clone, Copy, PartialEq)]

pub enum ValueType {
    Bool,
    Number,
    String,
    Object,
    None,
}

#[derive(Clone, Copy)]

pub struct Value {
    value_type: ValueType,
    casts: ValueUnion,
}

impl Value {
    #[inline]
    const fn new(value_type: ValueType, casts: ValueUnion) -> Self {
        Self { value_type, casts }
    }

    #[inline]
    pub const fn get_type(&self) -> ValueType {
        self.value_type
    }

    value_ctor!(bool, bool, bool, ValueType::Bool);
    value_ctor!(number, number, f64, ValueType::Number);
    value_ctor!(string, string, InternedString, ValueType::String);
    value_ctor!(object, object, HeapPtr<Object>, ValueType::Object);

    #[inline]
    pub const fn none() -> Self {
        Self::new(ValueType::None, ValueUnion { bool: false })
    }

    value_as_fn!(as_bool, bool, bool, ValueType::Bool);
    value_as_fn!(as_number, number, f64, ValueType::Number);
    value_as_fn!(as_string, string, InternedString, ValueType::String);
    value_as_fn!(as_object, object, HeapPtr<Object>, ValueType::Object);
}

impl Value {
    #[inline]
    pub fn is_falsey(&self) -> bool {
        match self.value_type {
            ValueType::Bool => !self.as_bool(),
            ValueType::Number => self.as_number() == 0.0,
            ValueType::String => self.as_string().is_empty(),
            ValueType::Object => false, // Objects are always truthy
            ValueType::None => true,
        }
    }
}

impl Add for Value {
    type Output = Result<Value, Error>;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        match self.value_type {
            ValueType::Number => match rhs.value_type {
                ValueType::Number => Ok(Self::number(self.as_number() + rhs.as_number())),
                _ => Err(Error::Runtime(
                    "Cannot add non-number types to numbers!".to_owned(),
                )),
            },
            ValueType::Bool => Err(Error::Runtime("Cannot add boolean values!".to_owned())),
            ValueType::String => unreachable!(),
            ValueType::Object => Err(Error::Runtime("Cannot add Object types!".to_owned())),
            ValueType::None => Err(Error::Runtime("Cannot add None types!".to_owned())),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, Error>;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        match self.value_type {
            ValueType::Number => match rhs.value_type {
                ValueType::Number => Ok(Self::number(self.as_number() - rhs.as_number())),
                _ => Err(Error::Runtime(
                    "Cannot subtract non-number types to numbers!".to_owned(),
                )),
            },
            ValueType::Bool => Err(Error::Runtime("Cannot subtract boolean values!".to_owned())),
            ValueType::String => unreachable!(),
            ValueType::Object => Err(Error::Runtime("Cannot subtract Object types!".to_owned())),
            ValueType::None => Err(Error::Runtime("Cannot subtract None types!".to_owned())),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, Error>;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        match self.value_type {
            ValueType::Number => match rhs.value_type {
                ValueType::Number => Ok(Self::number(self.as_number() * rhs.as_number())),
                _ => Err(Error::Runtime(
                    "Cannot multiply non-number types to numbers!".to_owned(),
                )),
            },
            ValueType::Bool => Err(Error::Runtime("Cannot multiply boolean values!".to_owned())),
            ValueType::String => unreachable!(),
            ValueType::Object => Err(Error::Runtime("Cannot multiply Object types!".to_owned())),
            ValueType::None => Err(Error::Runtime("Cannot multiply None types!".to_owned())),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, Error>;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        match self.value_type {
            ValueType::Number => match rhs.value_type {
                ValueType::Number => Ok(Self::number(self.as_number() / rhs.as_number())),
                _ => Err(Error::Runtime(
                    "Cannot divide non-number types to numbers!".to_owned(),
                )),
            },
            ValueType::Bool => Err(Error::Runtime("Cannot divide boolean values!".to_owned())),
            ValueType::String => unreachable!(),
            ValueType::Object => Err(Error::Runtime("Cannot divide Object types!".to_owned())),
            ValueType::None => Err(Error::Runtime("Cannot divide None types!".to_owned())),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value, Error>;

    #[inline]
    fn neg(self) -> Self::Output {
        match self.value_type {
            ValueType::Number => Ok(Self::number(-self.as_number())),
            ValueType::Bool => Err(Error::Runtime("Cannot negate boolean values!".to_owned())),
            ValueType::String => Err(Error::Runtime("Cannot negate String types!".to_owned())),
            ValueType::Object => Err(Error::Runtime("Cannot negate Object types!".to_owned())),
            ValueType::None => Err(Error::Runtime("Cannot negate None types!".to_owned())),
        }
    }
}

impl PartialEq for Value {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match self.value_type {
            ValueType::Number => match other.value_type {
                ValueType::Number => self.as_number() == other.as_number(),
                _ => false,
            },
            ValueType::Bool => match other.value_type {
                ValueType::Bool => self.as_bool() == other.as_bool(),
                _ => false,
            },
            ValueType::String => match other.value_type {
                ValueType::String => self.as_string() == self.as_string(),
                _ => false,
            },
            ValueType::Object => match other.value_type {
                ValueType::Object => self.as_object() == other.as_object(),
                _ => false,
            },
            ValueType::None => other.value_type == ValueType::None,
        }
    }
}

impl PartialOrd for Value {
    #[inline]
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
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value_type {
            ValueType::Number => Display::fmt(&self.as_number(), f),
            ValueType::Bool => Display::fmt(&self.as_bool(), f),
            ValueType::String => Display::fmt(&self.as_string(), f),
            ValueType::Object => Display::fmt(&self.as_object(), f),
            ValueType::None => f.write_str("None"),
        }
    }
}

impl Debug for Value {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value_type {
            ValueType::Number => f.write_fmt(format_args!("[Number] = {}", self.as_number())),
            ValueType::Bool => f.write_fmt(format_args!("[Bool] = {}", self.as_bool())),
            ValueType::String => f.write_fmt(format_args!("[String] = \"{}\"", self.as_string())),
            ValueType::Object => Debug::fmt(&self.as_object(), f),
            ValueType::None => f.write_str("None"),
        }
    }
}
