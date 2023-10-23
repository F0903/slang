use chunk::Chunk;
use debug::disassemble_chunk;
use opcode::OpCode;

mod chunk;
mod debug;
mod dynarray;
mod encoding;
mod memory;
mod opcode;
mod value;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut chunk = Chunk::new();
    chunk.write_constant(69.420, 1);
    chunk.write_constant(1234.0, 2);
    chunk.write_constant(1634.0, 3);
    chunk.write_opcode(OpCode::Return, 4);
    chunk.write_opcode(OpCode::Return, 3);

    chunk.line_numbers_map.encode_all();

    disassemble_chunk(&mut chunk, "TEST");
    Ok(())
}
