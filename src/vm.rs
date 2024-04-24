#[cfg(debug_assertions)]
use crate::debug::disassemble_instruction;
use {
    crate::{chunk::Chunk, compiler::Compiler, opcode::OpCode, stack::Stack, value::Value},
    std::{error::Error, ffi::CString, fmt::Display},
};

#[derive(Debug)]
pub enum InterpretError {
    CompileTime(String),
    Runtime(String),
}

impl Display for InterpretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompileTime(msg) => {
                f.write_fmt(format_args!("Encountered compile-time error.\n{}", msg))
            }
            Self::Runtime(msg) => f.write_fmt(format_args!("Encountered runtime error.\n{}", msg)),
        }
    }
}

impl Error for InterpretError {}

type InterpretResult = Result<(), InterpretError>;

macro_rules! binary_op_result {
    ($stack: expr, $op: tt) => {{
        let b = $stack.pop();
        let a = $stack.pop();
        println!("BINARY_OP: {} {} {}", a, stringify!($op), b);
        a $op b
    }};
}

macro_rules! binary_op_try {
    ($stack: expr, $op: tt) => {{
        let val = binary_op_result!($stack, $op)?;
        $stack.push(val);
    }};
}

macro_rules! binary_op_from_bool {
    ($stack: expr, $op: tt) => {{
        let val = binary_op_result!($stack, $op);
        $stack.push(Value::boolean(val));
    }};
}

pub struct VM {
    ip: *mut u8,
    stack: Stack<Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            ip: std::ptr::null_mut(),
            stack: Stack::new(),
        }
    }

    fn read_byte(&mut self) -> u8 {
        unsafe {
            let val = self.ip.read();
            self.ip = self.ip.add(1);
            val
        }
    }

    fn read_long(&mut self) -> u32 {
        unsafe {
            let val = self.ip.cast::<u32>().read();
            self.ip = self.ip.add(4);
            val
        }
    }

    fn read_constant_long(&mut self, chunk: &mut Chunk) -> Value {
        let index = self.read_long();
        chunk.get_constant(index)
    }

    fn read_constant(&mut self, chunk: &mut Chunk) -> Value {
        let index = self.read_byte();
        chunk.get_constant(index as u32)
    }

    pub fn interpret(&mut self, source: impl Into<Vec<u8>>) -> InterpretResult {
        let source = CString::new(source).map_err(|_| {
            InterpretError::CompileTime("Could not create a CString from source!".to_owned())
        })?;

        let mut compiler = Compiler::new();
        let chunk = compiler
            .compile(source.as_bytes())
            .map_err(|e| InterpretError::CompileTime(e.to_string()))?;

        self.ip = chunk.borrow().get_code_ptr();

        loop {
            #[cfg(debug_assertions)]
            {
                print!("{:?}", &self.stack);
                unsafe {
                    let offset = self.ip.offset_from(chunk.borrow().get_code_ptr());
                    disassemble_instruction(&mut chunk.borrow_mut(), offset as usize);
                }
            }

            let instruction = self.read_byte();
            match instruction.into() {
                OpCode::ConstantLong => {
                    let constant = self.read_constant_long(&mut chunk.borrow_mut());
                    self.stack.push(constant);
                }
                OpCode::Constant => {
                    let constant = self.read_constant(&mut chunk.borrow_mut());
                    self.stack.push(constant);
                }
                OpCode::None => self.stack.push(Value::none()),
                OpCode::True => self.stack.push(Value::boolean(true)),
                OpCode::False => self.stack.push(Value::boolean(false)),
                OpCode::Is => binary_op_from_bool!(self.stack, ==),
                OpCode::IsNot => binary_op_from_bool!(self.stack, !=),
                OpCode::Greater => binary_op_from_bool!(self.stack, >),
                OpCode::GreaterEqual => binary_op_from_bool!(self.stack, >=),
                OpCode::Less => binary_op_from_bool!(self.stack, <),
                OpCode::LessEqual => binary_op_from_bool!(self.stack, <=),
                OpCode::Add => binary_op_try!(self.stack, +),
                OpCode::Subtract => binary_op_try!(self.stack, -),
                OpCode::Multiply => binary_op_try!(self.stack, *),
                OpCode::Divide => binary_op_try!(self.stack, /),
                OpCode::Not => {
                    let val = self.stack.pop();
                    self.stack.push(Value::boolean(val.is_falsey()));
                }
                OpCode::Negate => {
                    let val = self.stack.get_top_mut_ref();
                    *val = (-(*val).clone())?;
                }
                OpCode::Return => {
                    println!("{}", self.stack.pop());
                    return Ok(());
                }
            }
        }
    }
}
