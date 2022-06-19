use super::Contextable;
use super::Function;
use super::NativeFunction;
use super::Result;
use super::VmContext;
use crate::line_reader::LineReader;
use crate::parser::Parser;
use crate::types::Argument;
use crate::types::Parameter;
use crate::types::ScriptFunction;
use std::fs::File;
use std::io::BufReader;

pub struct VirtualMachine {
    context: VmContext,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            context: VmContext::default(),
        }
    }

    pub fn get_context(&mut self) -> &mut VmContext {
        &mut self.context
    }

    fn match_args_to_params(args: &mut [Argument], params: &[Parameter]) {
        for arg in args.iter_mut() {
            for param in params.iter() {
                if arg.index == param.index {
                    arg.matched_name = Some(param.name.clone());
                }
            }
        }
    }

    fn call_script_func(&mut self, func: ScriptFunction, args: &mut [Argument]) -> Result<()> {
        Self::match_args_to_params(args, &func.params);
        Parser::parse_func_code(func.body.code, args, self)?;
        Ok(())
    }

    fn call_native_func(&self, func: NativeFunction, args: &mut [Argument]) -> Result<()> {
        Self::match_args_to_params(args, &func.params);
        func.call(args.to_owned());
        Ok(())
    }

    pub fn call_func(&mut self, name: impl AsRef<str>, args: &mut [Argument]) -> Result<()> {
        let name = name.as_ref();
        let func = self
            .context
            .get_func(name)
            .ok_or(format!("Could not call func {}! Does not exist.", name))?
            .clone();

        match func {
            Function::Native(x) => self.call_native_func(x, args),
            Function::Script(x) => self.call_script_func(x, args),
        }
    }

    fn execute(&mut self, reader: LineReader) -> Result<()> {
        Parser::parse_buffer(reader, self)?;
        Ok(())
    }

    pub fn register_native_func(&mut self, func: NativeFunction) {
        self.context.push_func(Function::Native(func));
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
