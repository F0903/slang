use crate::operators::Operation;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait NamedValue {
    fn get_name(&self) -> String;
    fn get_value(&self) -> Value;
    fn set_value(&mut self, val: Value);
}

impl NamedValue for Argument {
    fn get_name(&self) -> String {
        self.matched_name.as_ref().unwrap().clone() // Should always contain a value when this should be called.
    }

    fn get_value(&self) -> Value {
        self.value.clone()
    }

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

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(u32),
    Boolean(bool),
    Any,
    None,
}

impl Value {
    pub fn from_string(string: &str) -> Result<Self> {
        let chars = &mut string.chars();
        let first_char = chars.next().ok_or("Could not get first char of value.")?;

        if first_char == '"' && chars.last().ok_or("Could not get last char of value.")? == '"' {
            // Don't include the '"', so exlude the first and last char.
            return Ok(Value::String(string[1..string.len() - 1].to_string()));
        }

        if chars.all(|ch| ch.is_numeric()) {
            return Ok(Value::Number(string.parse()?));
        }

        if let Ok(x) = string.parse::<bool>() {
            return Ok(Value::Boolean(x));
        }

        Err("Invalid value.".into())
    }

    fn add(&self, other: &Value) -> Result<Self> {
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

    fn minus(&self, other: &Value) -> Result<Self> {
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

    fn multiply(&self, other: &Value) -> Result<Self> {
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

    fn divide(&self, other: &Value) -> Result<Self> {
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

    pub fn perform_op(&self, op: &Operation, other: &Value) -> Result<Self> {
        match op {
            Operation::Plus(_) => self.add(other),
            Operation::Minus(_) => self.minus(other),
            Operation::Multiply(_) => self.multiply(other),
            Operation::Divide(_) => self.divide(other),
            Operation::NoOp(_) => Ok(self.clone()),
        }
    }
}
