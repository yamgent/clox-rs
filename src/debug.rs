use std::io;

use crate::chunk::{Chunk, OpCode};

pub fn is_debug_trace_execution_enabled() -> bool {
    match std::env::var("DEBUG_TRACE_EXECUTION") {
        Ok(value) => value == "1",
        Err(_) => false,
    }
}

pub fn is_debug_print_code_enabled() -> bool {
    match std::env::var("DEBUG_PRINT_CODE") {
        Ok(value) => value == "1",
        Err(_) => false,
    }
}

pub fn disassemble_chunk<S: AsRef<str>, W: io::Write>(w: &mut W, chunk: &Chunk, name: S) {
    writeln!(w, "== {} ==", name.as_ref()).expect("writable");

    let mut offset = 0;
    while offset < chunk.code_len() {
        offset = disassemble_instruction(w, chunk, offset);
    }
}

pub fn disassemble_instruction<W: io::Write>(w: &mut W, chunk: &Chunk, offset: usize) -> usize {
    write!(w, "{:04} ", offset).expect("writable");

    if offset > 0 && chunk.get_line(offset) == chunk.get_line(offset - 1) {
        write!(w, "   | ").expect("writable");
    } else {
        write!(w, "{:4} ", chunk.get_line(offset)).expect("writable");
    }

    let instruction = chunk.get_code(offset);
    match OpCode::try_from(instruction) {
        Ok(code) => match code {
            OpCode::Return => simple_instruction(w, "OP_RETURN", offset),
            OpCode::Constant => constant_instruction(w, "OP_CONSTANT", chunk, offset),
            OpCode::Negate => simple_instruction(w, "OP_NEGATE", offset),
            OpCode::Add => simple_instruction(w, "OP_ADD", offset),
            OpCode::Subtract => simple_instruction(w, "OP_SUBTRACT", offset),
            OpCode::Multiply => simple_instruction(w, "OP_MULTIPLY", offset),
            OpCode::Divide => simple_instruction(w, "OP_DIVIDE", offset),
            OpCode::Nil => simple_instruction(w, "OP_NIL", offset),
            OpCode::True => simple_instruction(w, "OP_TRUE", offset),
            OpCode::False => simple_instruction(w, "OP_FALSE", offset),
            OpCode::Not => simple_instruction(w, "OP_NOT", offset),
            OpCode::Equal => simple_instruction(w, "OP_EQUAL", offset),
            OpCode::Greater => simple_instruction(w, "OP_GREATER", offset),
            OpCode::Less => simple_instruction(w, "OP_LESS", offset),
        },
        Err(_) => {
            writeln!(w, "Unknown opcode {}", instruction).expect("writable");
            offset + 1
        }
    }
}

fn simple_instruction<S: AsRef<str>, W: io::Write>(w: &mut W, name: S, offset: usize) -> usize {
    writeln!(w, "{}", name.as_ref()).expect("writable");
    offset + 1
}

fn constant_instruction<S: AsRef<str>, W: io::Write>(
    w: &mut W,
    name: S,
    chunk: &Chunk,
    offset: usize,
) -> usize {
    let constant = chunk.get_code(offset + 1);
    writeln!(
        w,
        "{:<16} {:4} '{:?}'",
        name.as_ref(),
        constant,
        chunk.constants().get(constant as usize)
    )
    .expect("writable");
    offset + 2
}

#[cfg(test)]
mod tests {
    use crate::value::Value;

    use super::*;

    #[test]
    fn test_disassemble_chunk_and_instructions() {
        {
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(1.2));
            chunk.write(OpCode::Constant as u8, 123);
            chunk.write(constant as u8, 123);

            let constant = chunk.constants_mut().add(Value::Number(3.4));
            chunk.write(OpCode::Constant as u8, 123);
            chunk.write(constant as u8, 123);

            chunk.write(OpCode::Add as u8, 123);

            let constant = chunk.constants_mut().add(Value::Number(5.6));
            chunk.write(OpCode::Constant as u8, 123);
            chunk.write(constant as u8, 123);

            chunk.write(OpCode::Divide as u8, 123);
            chunk.write(OpCode::Negate as u8, 123);

            chunk.write(OpCode::Return as u8, 123);

            chunk.write(OpCode::Subtract as u8, 124);
            chunk.write(OpCode::Multiply as u8, 125);
            chunk.write(255, 125); // invalid opcode

            let mut output = Vec::new();
            disassemble_chunk(&mut output, &chunk, "test chunk");

            assert_eq!(
                String::from_utf8(output)
                    .expect("valid utf8")
                    .lines()
                    .collect::<Vec<_>>(),
                vec![
                    "== test chunk ==",
                    "0000  123 OP_CONSTANT         0 'Number(1.2)'",
                    "0002    | OP_CONSTANT         1 'Number(3.4)'",
                    "0004    | OP_ADD",
                    "0005    | OP_CONSTANT         2 'Number(5.6)'",
                    "0007    | OP_DIVIDE",
                    "0008    | OP_NEGATE",
                    "0009    | OP_RETURN",
                    "0010  124 OP_SUBTRACT",
                    "0011  125 OP_MULTIPLY",
                    "0012    | Unknown opcode 255"
                ],
            );
        }

        {
            let mut chunk = Chunk::new();

            chunk.write(OpCode::Nil as u8, 123);
            chunk.write(OpCode::True as u8, 123);
            chunk.write(OpCode::False as u8, 123);
            chunk.write(OpCode::Not as u8, 123);
            chunk.write(OpCode::Equal as u8, 123);
            chunk.write(OpCode::Greater as u8, 123);
            chunk.write(OpCode::Less as u8, 123);

            let mut output = Vec::new();
            disassemble_chunk(&mut output, &chunk, "test chunk");

            assert_eq!(
                String::from_utf8(output)
                    .expect("valid utf8")
                    .lines()
                    .collect::<Vec<_>>(),
                vec![
                    "== test chunk ==",
                    "0000  123 OP_NIL",
                    "0001    | OP_TRUE",
                    "0002    | OP_FALSE",
                    "0003    | OP_NOT",
                    "0004    | OP_EQUAL",
                    "0005    | OP_GREATER",
                    "0006    | OP_LESS",
                ],
            );
        }
    }
}
