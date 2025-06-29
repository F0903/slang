// Each "disassembly function" returns how many byte long its instruction is

use crate::{compiler::chunk::Chunk, vm::opcode::OpCode};

fn simple_instruction(instruction: &OpCode) -> usize {
    print!("{:?}", instruction);
    1
}

#[allow(dead_code)]
fn constant_instruction(instruction: &OpCode, chunk: &Chunk, offset: usize) -> usize {
    let constant_index = chunk.read_byte(offset + 1);
    let constant_value = chunk.get_constant(constant_index as u32);
    print!("{:?}[{}] = {}", instruction, constant_index, constant_value);
    2
}

fn quad_constant_instruction(instruction: &OpCode, chunk: &Chunk, offset: usize) -> usize {
    let constant_index = chunk.read_quad(offset + 1);
    let constant_value = chunk.get_constant(constant_index);
    print!("{:?}[{}] = {}", instruction, constant_index, constant_value);
    5
}

fn byte_instruction(instruction: &OpCode, chunk: &Chunk, offset: usize) -> usize {
    let arg = chunk.read_byte(offset + 1);
    print!("{:?}[{:?}]", instruction, arg);
    2
}

fn double_instruction(instruction: &OpCode, chunk: &Chunk, offset: usize) -> usize {
    let arg = chunk.read_double(offset + 1);
    print!("{:?}[{:?}]", instruction, arg);
    3
}

fn jump_instruction(instruction: &OpCode, chunk: &Chunk, offset: usize, sign: isize) -> usize {
    let jump = chunk.read_double(offset + 1);
    let destination = offset as isize + 3 + (sign * jump as isize);
    print!("{:?} {:?} -> {:?}", instruction, offset, destination);
    3
}

fn closure_instruction(chunk: &Chunk, offset: usize) -> usize {
    let constant_index = chunk.read_double(offset + 1);
    let constant = chunk.get_constant(constant_index as u32);
    print!(
        "{:?}[{:?}] -> {:?}",
        OpCode::Closure,
        constant_index,
        constant
    );

    let function = constant.as_object().as_function();
    let mut upvalue_bytes = 0;
    for _ in 0..function.get_upvalue_count() {
        let is_local = chunk.read_byte(offset + 2) != 0;
        let index = chunk.read_double(offset + 3);
        upvalue_bytes += 3;
        print!(
            "{:0>4}\t| {} {}",
            offset + 2 + upvalue_bytes - 3,
            if is_local { "local" } else { "upvalue" },
            index
        );
    }

    3 + upvalue_bytes
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
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
        OpCode::Closure => closure_instruction(chunk, offset),
        OpCode::Backjump => jump_instruction(&opcode, chunk, offset, -1),
        OpCode::Jump | OpCode::JumpIfFalse | OpCode::JumpIfTrue => {
            jump_instruction(&opcode, chunk, offset, 1)
        }
        OpCode::Call => byte_instruction(&opcode, chunk, offset),
        OpCode::PopN
        | OpCode::GetLocal
        | OpCode::SetLocal
        | OpCode::GetUpvalue
        | OpCode::SetUpvalue
        | OpCode::CloseUpvalue => double_instruction(&opcode, chunk, offset),
        OpCode::DefineGlobal | OpCode::SetGlobal | OpCode::GetGlobal | OpCode::Constant => {
            quad_constant_instruction(&opcode, chunk, offset)
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

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("=== {} ===", name);

    let mut offset = 0;
    let count = chunk.get_bytes_count() - 1;
    while offset <= count {
        offset += disassemble_instruction(chunk, offset);
    }
    println!()
}
