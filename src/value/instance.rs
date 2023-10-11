use super::{Class, Value};
use crate::{error::Result, error::RuntimeError, token::Token};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone)]
pub struct Instance {
    class: Class,
    fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Value> {
        self.fields
            .get(&name.lexeme)
            .map(|x| x.clone())
            .ok_or_else(|| {
                RuntimeError::new(
                    name.clone(),
                    format!("Undefined property '{}'.", name.lexeme),
                )
            })
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields.insert(name.lexeme.clone(), value);
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} instance", self.class.name))
    }
}
