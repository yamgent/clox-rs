mod chunk;
mod debug;
mod value;

use crate::chunk::{Chunk, OpCode};

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.constants_mut().add(1.2);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::Return as u8, 123);

    debug::disassemble_chunk(&chunk, "test chunk");
}
