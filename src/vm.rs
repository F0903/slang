use std::{error::Error, fmt::Display};

use crate::{
    chunk::Chunk,
    debug::{disassemble_chunk, disassemble_instruction},
    light_stack::LightStack,
    opcode::OpCode,
    value::Value,
};

#[derive(Debug)]
pub enum InterpretError {
    CompileTime,
    Runtime,
}

impl Display for InterpretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompileTime => f.write_str("Encountered compile-time error."),
            Self::Runtime => f.write_str("Encountered runtime error."),
        }
    }
}

impl Error for InterpretError {}

type InterpretResult = Result<(), InterpretError>;

macro_rules! binary_op {
    ($stack: expr, $op: tt) => {{
        let b = $stack.pop();
        let a = $stack.pop();
        println!("BINARY_OP: {} {} {}", a, stringify!($op), b);
        $stack.push(a $op b);
    }};
}

pub struct VM {
    ip: *mut u8,
    stack: LightStack<Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            ip: std::ptr::null_mut(),
            stack: LightStack::new(),
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

    fn run(&mut self, chunk: &mut Chunk) -> InterpretResult {
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                print!("{:?}", &self.stack);
                unsafe {
                    let offset = self.ip.offset_from(chunk.get_code_ptr());
                    disassemble_instruction(chunk, offset as usize);
                }
            }

            let instruction = self.read_byte();
            match instruction.into() {
                OpCode::ConstantLong => {
                    let constant = self.read_constant_long(chunk);
                    self.stack.push(constant);
                }
                OpCode::Constant => {
                    let constant = self.read_constant(chunk);
                    self.stack.push(constant);
                }
                OpCode::Add => binary_op!(self.stack, +),
                OpCode::Subtract => binary_op!(self.stack, -),
                OpCode::Multiply => binary_op!(self.stack, *),
                OpCode::Divide => binary_op!(self.stack, /),
                OpCode::Negate => {
                    let val = self.stack.get_top_mut_ref();
                    *val = -*val;
                }
                OpCode::Return => {
                    println!("{}", self.stack.pop());
                    return Ok(());
                }
            }
        }
    }

    pub fn interpret(&mut self, chunk: &mut Chunk) -> InterpretResult {
        self.ip = chunk.get_code_ptr();
        self.stack.init();
        self.run(chunk)
    }
}

impl Drop for VM {
    fn drop(&mut self) {}
}
