use chunk::Chunk;
use opcode::OpCode;
use vm::VM;

mod chunk;
mod debug;
mod dynarray;
mod encoding;
mod light_stack;
mod memory;
mod opcode;
mod value;
mod vm;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut vm = VM::new();

    let chunk_start = std::time::Instant::now();
    let mut chunk = Chunk::new();
    chunk.write_constant(1234.0, 2);
    chunk.write_constant(1634.0, 2);
    chunk.write_opcode(OpCode::Add, 2);
    chunk.write_constant(2.0, 2);
    chunk.write_opcode(OpCode::Multiply, 2);
    chunk.write_constant(1000.0, 2);
    chunk.write_opcode(OpCode::Subtract, 2);
    chunk.write_constant(3.0, 2);
    chunk.write_opcode(OpCode::Divide, 2);
    chunk.write_opcode(OpCode::Negate, 2);
    chunk.write_opcode(OpCode::Return, 3);
    chunk.encode();
    let chunk_end = std::time::Instant::now();

    let vm_start = std::time::Instant::now();
    vm.interpret(&mut chunk)?;
    let vm_end = std::time::Instant::now();

    println!();
    println!(
        "Chunk writing took: {}us",
        (chunk_end - chunk_start).as_micros()
    );
    println!("Vm interpret took: {}us", (vm_end - vm_start).as_micros());

    Ok(())
}
