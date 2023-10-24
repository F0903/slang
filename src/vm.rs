use std::{error::Error, fmt::Display};

use crate::{
    chunk::Chunk,
    debug::{disassemble_chunk, disassemble_instruction},
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

pub struct VM {
    ip: *mut u8,
}

impl VM {
    pub fn new() -> Self {
        Self {
            ip: std::ptr::null_mut(),
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
            //#[cfg(feature = "debug_trace_execution")]
            unsafe {
                let offset = self.ip.offset_from(chunk.get_code_ptr());
                disassemble_instruction(chunk, offset as usize);
            }

            let instruction = self.read_byte();
            match instruction.into() {
                OpCode::ConstantLong => {
                    let constant = self.read_constant_long(chunk);
                    println!("{}", constant);
                }
                OpCode::Constant => {
                    let constant = self.read_constant(chunk);
                    println!("{}", constant);
                }
                OpCode::Return => return Ok(()),
            }
        }
    }

    pub fn interpret(&mut self, chunk: &mut Chunk) -> InterpretResult {
        self.ip = chunk.get_code_ptr();
        self.run(chunk)
    }
}

impl Drop for VM {
    fn drop(&mut self) {}
}
