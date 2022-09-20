use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{Result, RuntimeError},
    token::Token,
    value::Value,
};

pub type Env = Rc<RefCell<Environment>>;

pub struct Environment {
    enclosing: Option<Env>,
    values: HashMap<String, Value>,
}

impl Into<Env> for Environment {
    fn into(self) -> Env {
        Rc::new(RefCell::new(self))
    }
}

impl<'a> Environment {
    pub fn new(enclosing: Option<Env>) -> Self {
        Self {
            enclosing: enclosing,
            values: HashMap::new(),
        }
    }

    fn undef_var_err<T>(name: &Token) -> Result<T> {
        let msg = format!("Undefined variable '{}'", &name.lexeme);
        Err(RuntimeError::new(name.clone(), msg))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<()> {
        let val = match self.values.get_mut(&name.lexeme) {
            Some(x) => x,
            None => {
                if let Some(x) = &mut self.enclosing {
                    x.borrow_mut().assign(name, value)?;
                    return Ok(());
                }
                return Self::undef_var_err(name);
            }
        };
        *val = value;
        Ok(())
    }

    pub fn get(&self, name: &Token) -> Result<Value> {
        match self.values.get(&name.lexeme) {
            Some(x) => Ok(x.clone()),
            None => {
                if let Some(x) = &self.enclosing {
                    match x.borrow().get(&name) {
                        Ok(x) => return Ok(x.clone()),
                        Err(_) => (),
                    }
                }
                Self::undef_var_err(&name)
            }
        }
    }
}
