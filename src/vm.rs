use crate::defs::{Function, Variable};
use crate::parser::Parser;
use crate::value::Value;
use std::fs::File;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct VmContext {
    vars: Vec<Variable>, // Use hashmap instead?
    funcs: Vec<Function>,
}

pub struct VirtualMachine {
    context: VmContext,
}

impl VmContext {
    pub fn contains_var(&self, var_name: impl AsRef<str>) -> bool {
        self.vars.iter().any(|x| var_name.as_ref() == x.name)
    }

    pub fn contains_func(&self, func_name: impl AsRef<str>) -> bool {
        self.funcs.iter().any(|x| func_name.as_ref() == x.name)
    }

    pub fn register_var(&mut self, var: Variable) {
        println!("Registering var - {:?}", var);
        self.vars.push(var);
    }

    pub fn register_func(&mut self, func: Function) {
        println!("Registering func - {:?}", func);
        self.funcs.push(func);
    }

    pub fn get_vars(&self) -> Vec<Variable> {
        self.vars.clone()
    }

    pub fn get_funcs(&self) -> Vec<Function> {
        self.funcs.clone()
    }

    pub fn set_var(&mut self, name: impl AsRef<str>, value: Value) -> Result<()> {
        let name = name.as_ref();
        let var = self
            .vars
            .iter_mut()
            .find(|x| x.name == name)
            .ok_or(format!("Could not find variable '{}'!", name))?;
        var.value = value;
        Ok(())
    }
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            context: VmContext {
                vars: vec![],
                funcs: vec![],
            },
        }
    }

    pub fn get_context(&mut self) -> &mut VmContext {
        &mut self.context
    }

    pub fn call_func(&self, name: impl AsRef<str>, args: &[Value]) -> Result<()> {
        let name = name.as_ref();
        let func = self
            .context
            .funcs
            .iter()
            .find(|x| x.name == name)
            .ok_or(format!("Could not find function '{}'!", name))?;
        Parser::parse_func_code(&func.body.code, args)?;
        Ok(())
    }

    fn execute(&mut self, reader: impl std::io::BufRead) -> Result<()> {
        Parser::parse(reader, self)?;
        Ok(())
    }

    pub fn execute_file(&mut self, path: impl AsRef<str>) -> Result<()> {
        let file = File::open(path.as_ref())?;
        let reader = std::io::BufReader::new(file);
        self.execute(reader)
    }

    pub fn execute_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        self.execute(text.as_ref().as_bytes())
    }
}
