mod chunk;
mod debug;
mod value;
mod vm;

use crate::{
    chunk::{Chunk, OpCode},
    vm::VM,
};

fn main() {
    let mut vm = VM::new();

    let mut chunk = Chunk::new();

    let constant = chunk.constants_mut().add(1.2);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    let constant = chunk.constants_mut().add(3.4);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::Add as u8, 123);

    let constant = chunk.constants_mut().add(5.6);
    chunk.write(OpCode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::Divide as u8, 123);
    chunk.write(OpCode::Negate as u8, 123);

    chunk.write(OpCode::Return as u8, 123);

    debug::disassemble_chunk(&chunk, "test chunk");

    vm.interpret(chunk);
}
