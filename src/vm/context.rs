use super::native_func::Function;
use super::NamedVal;
use super::Result;
use crate::types::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct VmContext {
    vars: RefCell<HashMap<String, NamedVal>>,
    funcs: RefCell<HashMap<String, Function>>,
}

impl VmContext {
    pub fn new(vars: HashMap<String, NamedVal>, funcs: HashMap<String, Function>) -> Self {
        VmContext {
            vars: RefCell::new(vars),
            funcs: RefCell::new(funcs),
        }
    }
}

impl Default for VmContext {
    fn default() -> Self {
        Self::new(HashMap::new(), HashMap::new())
    }
}

impl VmContext {
    pub fn push_var(&self, var: NamedVal) {
        let name = { var.borrow().get_name().to_string() };
        println!("Pushing var: {} = {:?}", name, var.borrow().get_value());
        self.vars.borrow_mut().insert(name, var);
    }

    pub fn get_var(&self, name: &str) -> Option<NamedVal> {
        self.vars.borrow_mut().get(name).map(Rc::clone)
    }

    pub fn set_var(&self, name: &str, value: Value) -> Result<()> {
        println!("Setting var: {} = {:?}", name, value);
        let name = name;
        let var = self
            .get_var(name)
            .ok_or(format!("Could not find variable '{}'!", name))?;
        let mut var = var.borrow_mut();
        var.set_value(value);
        Ok(())
    }

    pub fn get_func(&self, name: &str) -> Option<Function> {
        self.funcs.borrow().get(name).cloned()
    }

    pub fn push_func(&self, func: Function) {
        println!("Pushing func: {:?}", func);
        self.funcs
            .borrow_mut()
            .insert(func.get_name().to_owned(), func);
    }

    pub fn contains_var(&self, var_name: &str) -> bool {
        self.get_var(var_name).is_some()
    }

    pub fn contains_func(&self, func_name: &str) -> bool {
        self.get_func(func_name).is_some()
    }
}
