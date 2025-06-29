use core::panic;
use std::ptr::NonNull;

use crate::{collections::DynArray, dbg_println, value::Value, vm::opcode::OpCode};

/// The struct that holds executable code along with their linemap and constant values.
#[derive(Debug, Clone)]
pub struct Chunk {
    code: DynArray<u8>,
    constants: DynArray<Value>,
    line_numbers_map: DynArray<u32>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: DynArray::new(),
            constants: DynArray::new(),
            line_numbers_map: DynArray::new(), // Change this to some kind of encoding.
        }
    }

    #[inline]
    pub(crate) const fn get_code_ptr(&self) -> Option<NonNull<u8>> {
        self.code.get_raw_ptr()
    }

    #[inline]
    pub fn write_byte(&mut self, byte: u8, line_number: u32) {
        self.code.push(byte);
        self.line_numbers_map.push(line_number);
    }

    #[inline]
    pub fn write_ptr(&mut self, bytes: *const u8, count: usize, line: u32) {
        self.code.push_ptr(bytes, count);
        for _ in 0..count {
            self.line_numbers_map.push(line);
        }
    }

    #[inline]
    pub fn write_double(&mut self, short: u16, line: u32) {
        self.write_ptr((&raw const short).cast(), 2, line);
    }

    #[inline]
    pub fn write_quad(&mut self, long: u32, line: u32) {
        self.write_ptr((&raw const long).cast(), 4, line);
    }

    #[inline]
    pub fn write_opcode(&mut self, opcode: OpCode, line: u32) {
        dbg_println!("WRITING OP: {:?}", opcode);

        self.write_byte(opcode as u8, line)
    }

    #[inline]
    pub fn write_opcode_with_byte_arg(&mut self, opcode: OpCode, arg: u8, line: u32) {
        dbg_println!("WRITING OP WITH ARG: {:?} + {:?}", opcode, arg);

        self.write_byte(opcode as u8, line);
        self.write_byte(arg, line);
    }

    #[inline]
    pub fn write_opcode_with_double_arg(&mut self, opcode: OpCode, arg: u16, line: u32) {
        dbg_println!("WRITING OP WITH ARG: {:?} + {:?}", opcode, arg);

        self.write_byte(opcode as u8, line);
        self.write_double(arg, line);
    }

    #[inline]
    pub fn write_opcode_with_quad(&mut self, opcode: OpCode, arg: u32, line: u32) {
        dbg_println!("WRITING OP WITH LONG ARG: {:?} + {:?}", opcode, arg);

        self.write_byte(opcode as u8, line);
        self.write_quad(arg, line);
    }

    #[inline]
    pub fn read_byte(&self, index: usize) -> u8 {
        self.code.copy_read(index)
    }

    #[inline]
    pub fn read_double(&self, index: usize) -> u16 {
        // SAFETY: This is safe assuming there where no compiliation errors resulting in malformed code.
        unsafe { self.code.read_cast(index) }
    }

    #[inline]
    pub fn read_quad(&self, index: usize) -> u32 {
        // SAFETY: This is safe assuming there where no compiliation errors resulting in malformed code.
        unsafe { self.code.read_cast(index) }
    }

    #[inline]
    pub fn replace_last_op(&mut self, new_op: OpCode) {
        dbg_println!("REPLACING LAST OP WITH: {:?}", new_op);

        self.code.replace(self.code.get_count() - 1, new_op.into())
    }

    #[inline]
    pub const fn get_bytes_count(&self) -> usize {
        self.code.get_count()
    }

    /// Returns index of the added constant.
    #[inline]
    pub fn add_constant(&mut self, value: Value) -> u32 {
        self.constants.push(value);
        (self.constants.get_count() - 1) as u32
    }

    /// Returns index of the added constant.
    pub fn add_constant_with_op(&mut self, value: Value, line: u32) -> u32 {
        dbg_println!("WRITING CONSTANT: {:?}", value);

        let constant_index = self.add_constant(value);
        let const_count = self.constants.get_count();
        if const_count <= u32::MAX as usize {
            self.write_opcode(OpCode::Constant, line);
            self.write_quad(constant_index, line);
        } else if const_count > u32::MAX as usize {
            panic!("Cannot add more constants! (how the hell are you using so many???)")
        }
        constant_index
    }

    #[inline]
    pub fn get_constant(&self, index: u32) -> &Value {
        self.constants.get(index as usize)
    }

    #[inline]
    pub fn get_constants(&self) -> &[Value] {
        self.constants.as_slice()
    }

    #[inline]
    pub fn get_line_number(&self, index: usize) -> u32 {
        self.line_numbers_map.copy_read(index)
    }
}
