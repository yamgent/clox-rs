use crate::chunk::{Chunk, OpCode};

pub fn disassemble_chunk<S: AsRef<str>>(chunk: &Chunk, name: S) {
    println!("== {} ==", name.as_ref());

    let mut offset = 0;
    while offset < chunk.code_len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);

    if offset > 0 && chunk.get_line(offset) == chunk.get_line(offset - 1) {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.get_line(offset));
    }

    let instruction = chunk.get_code(offset);
    match OpCode::try_from(instruction) {
        Ok(code) => match code {
            OpCode::Return => simple_instruction("OP_RETURN", offset),
            OpCode::Constant => constant_instruction("OP_CONSTANT", chunk, offset),
        },
        Err(_) => {
            println!("Unknown opcode {}", instruction);
            offset + 1
        }
    }
}

fn simple_instruction<S: AsRef<str>>(name: S, offset: usize) -> usize {
    println!("{}", name.as_ref());
    offset + 1
}

fn constant_instruction<S: AsRef<str>>(name: S, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.get_code(offset + 1);
    println!(
        "{:<16} {:4} '{}'",
        name.as_ref(),
        constant,
        chunk.constants().get(constant as usize)
    );
    offset + 2
}
