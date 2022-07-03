use super::Function;
use super::NativeFunction;
use super::Result;
use super::VmContext;
use crate::code_reader::CodeReader;
use crate::parser::Parser;
use crate::parser::ScopeParseResult;
use crate::types::Argument;
use crate::types::Parameter;
use crate::types::ScriptFunction;
use crate::types::Value;
use std::fs::File;

pub struct VirtualMachine {
    context: VmContext,
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine {
            context: VmContext::default(),
        }
    }

    pub fn get_context(&self) -> &VmContext {
        &self.context
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

    fn call_script_func(&self, func: ScriptFunction, args: &mut [Argument]) -> Result<Value> {
        Self::match_args_to_params(args, &func.params);
        if let ScopeParseResult::Return(x) = Parser::parse_func_code(func.code, args, self)? {
            Ok(x)
        } else {
            Ok(Value::None)
        }
    }

    fn call_native_func(&self, func: NativeFunction, args: &mut [Argument]) -> Result<Value> {
        let value = func.call(args.to_owned());
        Ok(value)
    }

    pub fn call_func(&self, name: impl AsRef<str>, args: &mut [Argument]) -> Result<Value> {
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

    fn execute(&self, reader: CodeReader) -> Result<()> {
        Parser::parse_buffer(reader, self)?;
        Ok(())
    }

    pub fn register_native_func(&self, name: impl ToString, func: fn(Vec<Argument>) -> Value) {
        self.context
            .push_func(Function::Native(NativeFunction::new(name, func)));
    }

    pub fn execute_file(&self, path: impl AsRef<str>) -> Result<()> {
        let file = File::open(path.as_ref())?;
        self.execute(CodeReader::from_file(file))
    }

    pub fn execute_text(&self, text: impl AsRef<str>) -> Result<()> {
        self.execute(CodeReader::from_str(text))
    }
}
