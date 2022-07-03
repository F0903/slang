use crate::operators::Operation;
use std::{cmp::Ordering, fmt::Debug};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait NamedValue: Debug {
    fn get_name(&self) -> String;
    fn get_value(&self) -> Value;
    fn set_value(&mut self, val: Value);
}

impl NamedValue for Argument {
    #[inline]
    fn get_name(&self) -> String {
        self.matched_name.as_ref().unwrap().clone() // Should always contain a value when this should be called.
    }

    #[inline]
    fn get_value(&self) -> Value {
        self.value.clone()
    }

    #[inline]
    fn set_value(&mut self, val: Value) {
        self.value = val;
    }
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub matched_name: Option<String>,
    pub index: usize,
    pub value: Value,
}

impl Argument {
    pub fn new(index: usize, value: Value) -> Self {
        Self {
            matched_name: None,
            index,
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    None,
}

impl PartialEq for Value {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::String(x) => match other {
                Value::String(y) => x == y,
                _ => false,
            },
            Value::Number(x) => match other {
                Value::Number(y) => x == y,
                _ => false,
            },
            Value::Boolean(x) => match other {
                Value::Boolean(y) => x == y,
                _ => false,
            },
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Value::String(x) => match other {
                Value::String(y) => x.partial_cmp(y),
                _ => None,
            },
            Value::Number(x) => match other {
                Value::Number(y) => x.partial_cmp(y),
                _ => None,
            },
            Value::Boolean(x) => match other {
                Value::Boolean(y) => x.partial_cmp(y),
                _ => None,
            },
            _ => None,
        }
    }
}

impl Value {
    #[inline]
    pub fn from_string(string: &str) -> Result<Self> {
        let mut chars = string.chars();
        let first_char = chars.next().ok_or("Could not get first char of value.")?;
        if first_char == '"' && chars.last().ok_or("Could not get last char of value.")? == '"' {
            // Don't include the '"', so exlude the first and last char.
            return Ok(Value::String(string[1..string.len() - 1].to_string()));
        }

        if let Ok(x) = string.parse::<f64>() {
            return Ok(Value::Number(x));
        }

        if let Ok(x) = string.parse::<bool>() {
            return Ok(Value::Boolean(x));
        }

        Err(format!("Value '{}' is invalid. Either it's a variable that dosn't exist, or an incorrect litteral or expression.", string).into())
    }

    #[inline]
    fn add(&self, other: &Self) -> Result<Self> {
        let new_val = match self {
            Value::Number(x) => {
                let other = match other {
                    Value::Number(y) => y,
                    _ => return Err("Cannot add this value to a number.".into()),
                };
                Value::Number(x + other)
            }
            Value::String(x) => {
                let other = match other {
                    Value::String(y) => y.clone(),
                    Value::Number(y) => y.to_string(),
                    _ => return Err("Cannot add this value to a string.".into()),
                };
                let mut new_str = String::default();
                new_str.push_str(x);
                new_str.push_str(&other);
                Value::String(new_str)
            }
            _ => return Err("Cannot perform add on this value.".into()),
        };
        Ok(new_val)
    }

    #[inline]
    fn minus(&self, other: &Self) -> Result<Self> {
        let new_val = match self {
            Value::Number(x) => {
                let other = match other {
                    Value::Number(y) => y,
                    _ => return Err("Cannot subtract this value to a number.".into()),
                };
                Value::Number(x - other)
            }
            _ => return Err("Cannot perform subtract on this value.".into()),
        };
        Ok(new_val)
    }

    #[inline]
    fn multiply(&self, other: &Self) -> Result<Self> {
        let new_val = match self {
            Value::Number(x) => {
                let other = match other {
                    Value::Number(y) => y,
                    _ => return Err("Cannot multiply this value to a number.".into()),
                };
                Value::Number(x * other)
            }
            _ => return Err("Cannot perform multiply on this value.".into()),
        };
        Ok(new_val)
    }

    #[inline]
    fn divide(&self, other: &Self) -> Result<Self> {
        let new_val = match self {
            Value::Number(x) => {
                let other = match other {
                    Value::Number(y) => y,
                    _ => return Err("Cannot divide this value to a number.".into()),
                };
                Value::Number(x / other)
            }
            _ => return Err("Cannot perform divide on this value.".into()),
        };
        Ok(new_val)
    }

    #[inline]
    fn equal(&self, other: &Self) -> Result<Self> {
        let is_eq = self == other;
        Ok(Value::Boolean(is_eq))
    }

    #[inline]
    fn less_than(&self, other: &Self) -> Result<Self> {
        let is_less = self < other;
        Ok(Value::Boolean(is_less))
    }

    #[inline]
    fn less_or_eq(&self, other: &Self) -> Result<Self> {
        let less_or_eq = self <= other;
        Ok(Value::Boolean(less_or_eq))
    }

    #[inline]
    fn more_than(&self, other: &Self) -> Result<Self> {
        let more_than = self > other;
        Ok(Value::Boolean(more_than))
    }

    #[inline]
    fn more_or_eq(&self, other: &Self) -> Result<Self> {
        let more_or_eq = self >= other;
        Ok(Value::Boolean(more_or_eq))
    }

    #[inline]
    pub fn perform_op(&self, op: &Operation, other: &Value) -> Result<Value> {
        match op {
            Operation::Plus(_) => self.add(other),
            Operation::Minus(_) => self.minus(other),
            Operation::Multiply(_) => self.multiply(other),
            Operation::Divide(_) => self.divide(other),
            Operation::Equal(_) => self.equal(other),
            Operation::LessThan(_) => self.less_than(other),
            Operation::LessOrEq(_) => self.less_or_eq(other),
            Operation::MoreThan(_) => self.more_than(other),
            Operation::MoreOrEq(_) => self.more_or_eq(other),
            Operation::NoOp(_) => Ok(self.clone()),
        }
    }
}
