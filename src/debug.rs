use crate::{chunk::Chunk, opcode::OpCode};

fn handle_simple_instr(instruction: &OpCode) -> usize {
    print!("{:?}", instruction);
    1
}

fn handle_constant_instr(instruction: &OpCode, chunk: &mut Chunk, offset: usize) -> usize {
    let constant_index = chunk.read(offset + 1);
    let constant_value = chunk.get_constant(constant_index as u32);
    print!("{:?} {} = {}", instruction, constant_index, constant_value);
    2
}

fn handle_constant_long_instr(instruction: &OpCode, chunk: &mut Chunk, offset: usize) -> usize {
    let constant_index = chunk.read_long(offset + 1);
    let constant_value = chunk.get_constant(constant_index);
    print!("{:?} {} = {}", instruction, constant_index, constant_value);
    5
}

pub fn disassemble_instruction(chunk: &mut Chunk, offset: usize) -> usize {
    print!("{:0>4} ", offset);

    let line = chunk.get_line_number(offset);
    if offset > 0 && line == chunk.get_line_number(offset - 1) {
        print!("   | ");
    } else {
        print!("{:0>4} ", line);
    }

    let instruction = chunk.read(offset);
    let opcode = instruction.into();
    let inst_offset = match opcode {
        OpCode::DefineGlobal | OpCode::GetGlobal | OpCode::Constant => {
            handle_constant_long_instr(&opcode, chunk, offset)
        }
        OpCode::Return
        | OpCode::Pop
        | OpCode::Greater
        | OpCode::GreaterEqual
        | OpCode::Less
        | OpCode::LessEqual
        | OpCode::Is
        | OpCode::IsNot
        | OpCode::Not
        | OpCode::False
        | OpCode::True
        | OpCode::None
        | OpCode::Negate
        | OpCode::Add
        | OpCode::Subtract
        | OpCode::Multiply
        | OpCode::Divide => handle_simple_instr(&opcode),
    };
    println!();
    inst_offset
}

pub fn disassemble_chunk(chunk: &mut Chunk, name: &str) {
    println!("=== {} ===", name);

    let mut offset = 0;
    let count = chunk.get_instruction_count() - 1;
    while offset <= count {
        offset += disassemble_instruction(chunk, offset);
    }
    println!()
}
