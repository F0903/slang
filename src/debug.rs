use crate::{chunk::Chunk, opcode::OpCode};

fn handle_simple_instr(instruction: &OpCode, offset: usize) -> usize {
    print!("{:?}", instruction);
    println!();
    offset + 1
}

fn handle_constant_instr(instruction: &OpCode, chunk: &mut Chunk, offset: usize) -> usize {
    let constant_index = chunk.read(offset + 1);
    let constant_value = chunk.get_constant(constant_index as u32);
    print!("{:?} {} = {}", instruction, constant_index, constant_value);
    println!();
    offset + 2
}

fn handle_constant_long_instr(instruction: &OpCode, chunk: &mut Chunk, offset: usize) -> usize {
    let constant_index = chunk.read_long(offset + 1);
    let constant_value = chunk.get_constant(constant_index);
    print!("{:?} {} = {}", instruction, constant_index, constant_value);
    println!();
    offset + 5
}

fn disassemble_instruction(chunk: &mut Chunk, offset: usize) -> usize {
    print!("{:0>4} ", offset);

    let line = chunk.get_line_number(offset);
    if offset > 0 && line == chunk.get_line_number(offset - 1) {
        print!("   | ");
    } else {
        print!("{:0>4} ", line);
    }

    let instruction = chunk.read(offset);
    let opcode = instruction.into();
    match opcode {
        OpCode::ConstantLong => handle_constant_long_instr(&opcode, chunk, offset),
        OpCode::Constant => handle_constant_instr(&opcode, chunk, offset),
        OpCode::Return => handle_simple_instr(&opcode, offset),
    }
}

pub fn disassemble_chunk(chunk: &mut Chunk, name: &str) {
    println!("=== {} ===", name);

    let mut offset = 0;
    let count = chunk.get_instruction_count();
    loop {
        if offset >= count {
            break;
        }
        offset = disassemble_instruction(chunk, offset);
    }
}
