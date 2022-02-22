use crate::defs::{Function, Variable};
use crate::operators::Operation;
use crate::parser::{ParseResult, Parser};
use std::fs::File;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Vm {
    fn register_vars(&mut self, vars: Vec<Variable>);

    fn register_funcs(&mut self, funcs: Vec<Function>);
}

pub trait VmContext {
    fn get_vars(&self) -> Vec<Variable>;
    fn get_funcs(&self) -> Vec<Function>;

    fn contains_var(&self, var_name: &str);
    fn contains_func(&self, func_name: &str);

    fn perform_op(&mut self, op: Operation) -> Result<()>;
}

pub struct VmGlobalContext {
    global_vars: Vec<Variable>,
    global_funcs: Vec<Function>,
}

impl VmContext for VmGlobalContext {
    fn get_vars(&self) -> Vec<Variable> {
        self.global_vars.clone()
    }

    fn get_funcs(&self) -> Vec<Function> {
        self.global_funcs.clone()
    }

    fn contains_var(&self, var_name: &str) {
        self.global_vars.iter().any(|x| var_name == x.name);
    }

    fn contains_func(&self, func_name: &str) {
        self.global_funcs.iter().any(|x| func_name == x.name);
    }

    fn perform_op(&mut self, op: Operation) -> Result<()> {
        //match op {}
        Ok(())
    }
}

pub struct VirtualMachine {
    parser: Parser,
    context: VmGlobalContext,
}

impl Vm for VirtualMachine {
    fn register_vars(&mut self, vars: Vec<Variable>) {
        self.context.global_vars = vars;
    }

    fn register_funcs(&mut self, funcs: Vec<Function>) {
        self.context.global_funcs = funcs;
    }
}

impl VirtualMachine {
    pub fn new(parser: Parser) -> Self {
        VirtualMachine {
            context: VmGlobalContext {
                global_vars: vec![],
                global_funcs: vec![],
            },
            parser,
        }
    }

    fn register_parse_result(&mut self, parse: ParseResult) {
        self.register_vars(parse.vars);
        self.register_funcs(parse.funcs);
    }

    fn execute(&mut self, reader: impl std::io::BufRead) -> Result<()> {
        let parse = self.parser.parse(reader)?;
        self.register_parse_result(parse);
        Ok(())
    }

    pub fn execute_file(&mut self, path: &str) -> Result<()> {
        let file = File::open(path)?;
        let reader = std::io::BufReader::new(file);
        self.execute(reader)
    }

    pub fn execute_text(&mut self, text: &str) -> Result<()> {
        self.execute(text.as_bytes())
    }
}
