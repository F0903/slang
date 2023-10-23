use core::panic;
use std::ptr::addr_of;

use crate::{
    dynarray::DynArray,
    encoding::{self, EncodedDynArray},
    opcode::OpCode,
    value::Value,
};

pub struct Chunk {
    code: DynArray<u8>,
    constants: DynArray<Value>,
    pub line_numbers_map: EncodedDynArray<encoding::RLE>, //DEBUG
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: DynArray::new(),
            constants: DynArray::new(),
            line_numbers_map: EncodedDynArray::new(), // Change this to some kind of encoding.
        }
    }

    pub fn write_opcode(&mut self, opcode: OpCode, line_number: u32) {
        self.write(opcode as u8, line_number)
    }

    pub fn write(&mut self, byte: u8, line_number: u32) {
        self.code.write(byte);
        self.line_numbers_map.write(line_number);
    }

    pub fn write_long(&mut self, bytes: *const u8, count: usize, line_number: u32) {
        self.code.write_ptr(bytes, count);
        for _ in 0..count {
            self.line_numbers_map.write(line_number);
        }
    }

    pub const fn read(&self, offset: usize) -> u8 {
        self.code.read(offset)
    }

    pub const fn read_long(&self, offset: usize) -> u32 {
        self.code.read_cast(offset)
    }

    pub const fn get_instruction_count(&self) -> usize {
        self.code.get_count()
    }

    /// Returns index of the added constant.
    fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write(value);
        self.constants.get_count() - 1
    }

    /// Returns index of the added constant.
    pub fn write_constant(&mut self, value: Value, line: u32) -> u32 {
        let constant = self.add_constant(value);
        let trunc_constant = constant as u32;

        let const_count = self.constants.get_count();
        if const_count <= u8::MAX as usize {
            self.write_opcode(OpCode::Constant, line);
            self.write(constant as u8, line);
        } else if const_count >= u8::MAX as usize {
            self.write_opcode(OpCode::ConstantLong, line);
            self.write_long(addr_of!(trunc_constant) as *const u8, 4, line);
        } else if const_count > u32::MAX as usize {
            panic!("Cannot add more constants! (how the hell are you using so many???)")
        }
        trunc_constant
    }

    pub const fn get_constant(&self, index: u32) -> Value {
        self.constants.read(index as usize)
    }

    pub fn get_line_number(&mut self, index: usize) -> u32 {
        self.line_numbers_map.read(index)
    }
}