use crate::{chunk::Chunk, opcode::OpCode};

// Each "disassembly function" returns how many byte long its instruction is

fn simple_instruction(instruction: &OpCode) -> usize {
    print!("{:?}", instruction);
    1
}

fn constant_instruction(instruction: &OpCode, chunk: &mut Chunk, offset: usize) -> usize {
    let constant_index = chunk.read_byte(offset + 1);
    let constant_value = chunk.get_constant(constant_index as u32);
    print!("{:?} {} = {}", instruction, constant_index, constant_value);
    2
}

fn constant_long_instruction(instruction: &OpCode, chunk: &mut Chunk, offset: usize) -> usize {
    let constant_index = chunk.read_quad(offset + 1);
    let constant_value = chunk.get_constant(constant_index);
    print!("{:?} {} = {}", instruction, constant_index, constant_value);
    5
}

fn byte_instruction(instruction: &OpCode, chunk: &mut Chunk, offset: usize) -> usize {
    let arg = chunk.read_byte(offset + 1);
    print!("{:?}[{:?}]", instruction, arg);
    2
}

fn jump_instruction(instruction: &OpCode, chunk: &mut Chunk, offset: usize, sign: isize) -> usize {
    let jump = chunk.read_double(offset + 1);
    print!(
        "{:?} {:?} -> {:?}",
        instruction,
        offset,
        offset + 3 + (sign * jump as isize) as usize
    );
    3
}

pub fn disassemble_instruction(chunk: &mut Chunk, offset: usize) -> usize {
    print!("{:0>4} ", offset);

    let line = chunk.get_line_number(offset);
    if offset > 0 && line == chunk.get_line_number(offset - 1) {
        print!("   | ");
    } else {
        print!("{:0>4} ", line);
    }

    let instruction = chunk.read_byte(offset);
    let opcode = instruction.into();
    let inst_offset = match opcode {
        OpCode::Backjump => jump_instruction(&opcode, chunk, offset, -1),
        OpCode::Jump | OpCode::JumpIfFalse | OpCode::JumpIfTrue => {
            jump_instruction(&opcode, chunk, offset, 1)
        }
        OpCode::PopN | OpCode::GetLocal | OpCode::SetLocal => {
            byte_instruction(&opcode, chunk, offset)
        }
        OpCode::DefineGlobal | OpCode::SetGlobal | OpCode::GetGlobal | OpCode::Constant => {
            constant_long_instruction(&opcode, chunk, offset)
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
        | OpCode::Divide => simple_instruction(&opcode),
    };
    println!();
    inst_offset
}

pub fn disassemble_chunk(chunk: &mut Chunk, name: &str) {
    println!("=== {} ===", name);

    let mut offset = 0;
    let count = chunk.get_bytes_count() - 1;
    while offset <= count {
        offset += disassemble_instruction(chunk, offset);
    }
    println!()
}
