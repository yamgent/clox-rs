use crate::value::ValueArray;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    Return,
    Constant,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Nil,
    True,
    False,
    Not,
    Equal,
    Greater,
    Less,
    // remember to modify the following areas when adding
    // a new enum variant:
    //      - OpCode::try_from()
    //      - tests::test_opcode_try_from()
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::Return),
            1 => Ok(OpCode::Constant),
            2 => Ok(OpCode::Negate),
            3 => Ok(OpCode::Add),
            4 => Ok(OpCode::Subtract),
            5 => Ok(OpCode::Multiply),
            6 => Ok(OpCode::Divide),
            7 => Ok(OpCode::Nil),
            8 => Ok(OpCode::True),
            9 => Ok(OpCode::False),
            10 => Ok(OpCode::Not),
            11 => Ok(OpCode::Equal),
            12 => Ok(OpCode::Greater),
            13 => Ok(OpCode::Less),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Chunk {
    code: Vec<u8>,
    constants: ValueArray,
    lines: Vec<u32>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            constants: ValueArray::new(),
            lines: vec![],
        }
    }

    pub fn write(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn get_code(&self, i: usize) -> u8 {
        self.code[i]
    }

    pub fn get_line(&self, i: usize) -> u32 {
        self.lines[i]
    }

    pub fn code_len(&self) -> usize {
        self.code.len()
    }

    pub fn constants(&self) -> &ValueArray {
        &self.constants
    }

    pub fn constants_mut(&mut self) -> &mut ValueArray {
        &mut self.constants
    }
}

#[cfg(test)]
mod tests {
    use crate::value::Value;

    use super::*;

    #[test]
    fn test_opcode_try_from() {
        [
            OpCode::Return,
            OpCode::Constant,
            OpCode::Negate,
            OpCode::Add,
            OpCode::Subtract,
            OpCode::Multiply,
            OpCode::Divide,
            OpCode::Nil,
            OpCode::True,
            OpCode::False,
            OpCode::Not,
            OpCode::Equal,
            OpCode::Greater,
            OpCode::Less,
        ]
        .into_iter()
        .for_each(|opcode| {
            assert_eq!(
                <u8 as TryInto::<OpCode>>::try_into(opcode as u8),
                Ok(opcode),
                "{:?}",
                opcode
            );
        });

        assert_eq!(<u8 as TryInto::<OpCode>>::try_into(255), Err(()));
    }

    #[test]
    fn test_chunk_write() {
        let mut chunk = Chunk::new();
        chunk.write(8, 155);
        chunk.write(9, 156);
        chunk.write(15, 156);
        chunk.write(2, 157);

        assert_eq!(chunk.code, vec![8, 9, 15, 2]);
        assert_eq!(chunk.lines, vec![155, 156, 156, 157]);

        assert_eq!(chunk.get_code(0), 8);
        assert_eq!(chunk.get_code(1), 9);
        assert_eq!(chunk.get_code(2), 15);
        assert_eq!(chunk.get_code(3), 2);

        assert_eq!(chunk.get_line(0), 155);
        assert_eq!(chunk.get_line(1), 156);
        assert_eq!(chunk.get_line(2), 156);
        assert_eq!(chunk.get_line(3), 157);

        assert_eq!(chunk.code_len(), 4);
    }

    #[test]
    fn test_chunk_constants() {
        let mut chunk = Chunk::new();
        let mut value_array = ValueArray::new();
        assert_eq!(chunk.constants(), &value_array);

        value_array.add(Value::Number(10.0));
        chunk.constants_mut().add(Value::Number(10.0));
        assert_eq!(chunk.constants(), &value_array);
    }
}
