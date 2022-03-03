use crate::defs::Function;
use crate::line_reader::LineReader;
use crate::parser::Parser;
use crate::value::{Argument, NamedValue, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use std::rc::Rc;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type NamedVal = Rc<RefCell<dyn NamedValue>>;

pub trait Contextable {
    fn get_var(&self, name: &str) -> Option<NamedVal>;
    fn get_func(&self, name: &str) -> Option<&Function>;

    fn push_var(&mut self, var: NamedVal); // Make this func more ergonomic to use. See if traits work with it.
    fn push_func(&mut self, func: Function);
}

pub trait ExecutionContext: Contextable {
    fn contains_var(&self, var_name: &str) -> bool;
    fn contains_func(&self, func_name: &str) -> bool;

    fn set_var(&mut self, name: &str, value: Value) -> Result<()>;
}

#[derive(Clone)]
pub struct VmContext {
    vars: HashMap<String, NamedVal>,
    funcs: HashMap<String, Function>,
}

pub struct VirtualMachine {
    context: VmContext,
}

impl Contextable for VmContext {
    fn get_var(&self, name: &str) -> Option<NamedVal> {
        self.vars.get(name).map(Rc::clone)
    }

    fn get_func(&self, name: &str) -> Option<&Function> {
        self.funcs.get(name)
    }

    fn push_var(&mut self, var: NamedVal) {
        let name;
        {
            name = var.borrow().get_name().to_string();
        }
        println!("Pushing var: {} = {:?}", name, var.borrow().get_value());
        self.vars.insert(name, var);
    }

    fn push_func(&mut self, func: Function) {
        println!("Pushing func: {:?}", func);
        self.funcs.insert(func.name.clone(), func);
    }
}

impl<T: Contextable> ExecutionContext for T {
    fn contains_var(&self, var_name: &str) -> bool {
        self.get_var(var_name).is_some()
    }

    fn contains_func(&self, func_name: &str) -> bool {
        self.get_func(func_name).is_some()
    }

    fn set_var(&mut self, name: &str, value: Value) -> Result<()> {
        println!("Setting var: {} = {:?}", name, value);
        let name = name;
        let var = self
            .get_var(name)
            .ok_or(format!("Could not find variable '{}'!", name))?;
        let mut var = var.borrow_mut();
        var.set_value(value);
        Ok(())
    }
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            context: VmContext {
                vars: HashMap::new(),
                funcs: HashMap::new(),
            },
        }
    }

    pub fn get_context(&mut self) -> &mut VmContext {
        &mut self.context
    }

    pub fn call_func(&mut self, name: impl AsRef<str>, args: &mut [Argument]) -> Result<()> {
        let name = name.as_ref();
        let func = self
            .context
            .get_func(name)
            .ok_or(format!("Could not call func {}! Does not exist.", name))?
            .clone();

        for arg in args.iter_mut() {
            for param in func.params.iter() {
                if arg.index == param.index {
                    arg.matched_name = Some(param.name.clone());
                }
            }
        }

        Parser::parse_func_code(func.body.code, args, self)?;
        Ok(())
    }

    fn execute(&mut self, reader: LineReader) -> Result<()> {
        Parser::parse_buffer(reader, self)?;
        Ok(())
    }

    pub fn execute_file(&mut self, path: impl AsRef<str>) -> Result<()> {
        let file = File::open(path.as_ref())?;
        let reader = std::io::BufReader::new(file);
        self.execute(LineReader::new(reader))
    }

    pub fn execute_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        let reader = BufReader::new(text.as_ref().as_bytes());
        self.execute(LineReader::new(reader))
    }
}
