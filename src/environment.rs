use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{Result, RuntimeError},
    token::Token,
    value::Value,
};

pub type EnvPtr = Rc<RefCell<Environment>>;

#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<EnvPtr>,
    values: HashMap<String, Value>,
}

impl Into<EnvPtr> for Environment {
    fn into(self) -> EnvPtr {
        Rc::new(RefCell::new(self))
    }
}

pub trait GetDeep {
    fn get_ancestor(&self, distance: u32) -> Self;

    fn get_at(&self, distance: u32, name: &str) -> Value;

    fn assign_at(&self, distance: u32, name: &Token, value: Value);
}

impl GetDeep for EnvPtr {
    fn get_ancestor(&self, distance: u32) -> Self {
        let mut environment = self.clone();
        for _ in 0..distance {
            if let Some(x) = environment.clone().borrow().enclosing.clone() {
                environment = x;
            }
        }
        environment
    }

    fn get_at(&self, distance: u32, name: &str) -> Value {
        self.get_ancestor(distance)
            .borrow()
            .values
            .get(name)
            .cloned()
            .expect(&format!(
                "Variable {} at scope {} not found! (internal error)",
                &name, distance
            ))
    }

    fn assign_at(&self, distance: u32, name: &Token, value: Value) {
        let ancestor = self.get_ancestor(distance);
        ancestor
            .borrow_mut()
            .values
            .insert(name.lexeme.clone(), value);
    }
}

impl<'a> Environment {
    pub fn new(enclosing: Option<EnvPtr>) -> Self {
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
