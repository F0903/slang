use core::panic;

use crate::{
    collections::{DynArray, EncodedDynArray},
    dbg_println, encoding,
    opcode::OpCode,
    value::Value,
};

pub struct Chunk {
    code: DynArray<u8>,
    constants: DynArray<Value>,
    line_numbers_map: EncodedDynArray<encoding::RLE>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: DynArray::new(None),
            constants: DynArray::new(None),
            line_numbers_map: EncodedDynArray::new(), // Change this to some kind of encoding.
        }
    }

    pub(crate) const fn get_code_ptr(&self) -> *mut u8 {
        self.code.get_raw_ptr()
    }

    pub fn write(&mut self, byte: u8, line_number: u32) {
        self.code.push(byte);
        self.line_numbers_map.write(line_number);
    }

    pub fn write_ptr(&mut self, bytes: *const u8, count: usize, line_number: u32) {
        self.code.push_ptr(bytes, count);
        for _ in 0..count {
            self.line_numbers_map.write(line_number);
        }
    }

    pub fn write_long(&mut self, long: u32, line_number: u32) {
        self.write_ptr(&raw const long as *const u8, 4, line_number);
    }

    pub fn write_opcode(&mut self, opcode: OpCode, line_number: u32) {
        dbg_println!("WRITING OP: {:?}", opcode);

        self.write(opcode as u8, line_number)
    }

    pub fn write_opcode_with_arg(&mut self, opcode: OpCode, arg: u8, line_number: u32) {
        dbg_println!("WRITING OP WITH ARG: {:?} + {:?}", opcode, arg);

        self.write(opcode as u8, line_number);
        self.write(arg, line_number);
    }

    pub fn write_opcode_with_long_arg(&mut self, opcode: OpCode, arg: u32, line_number: u32) {
        dbg_println!("WRITING OP WITH LONG ARG: {:?} + {:?}", opcode, arg);

        self.write(opcode as u8, line_number);
        self.write_long(arg, line_number);
    }

    pub fn read(&self, index: usize) -> u8 {
        *self.code.read(index)
    }

    pub fn replace_last_op(&self, new_op: OpCode) {
        dbg_println!("REPLACING LAST OP WITH: {:?}", new_op);

        self.code.replace(self.code.get_count() - 1, new_op.into())
    }

    pub fn read_long(&self, index: usize) -> u32 {
        self.code.read_cast(index)
    }

    pub const fn get_bytes_count(&self) -> usize {
        self.code.get_count()
    }

    /// Returns index of the added constant.
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
            self.write_long(constant_index, line);
        } else if const_count > u32::MAX as usize {
            panic!("Cannot add more constants! (how the hell are you using so many???)")
        }
        constant_index
    }

    pub fn get_constant(&self, index: u32) -> &Value {
        self.constants.read(index as usize)
    }

    pub fn get_line_number(&mut self, index: usize) -> u32 {
        self.line_numbers_map.read(index)
    }

    pub fn encode(&mut self) {
        self.line_numbers_map.encode_all();
    }
}
