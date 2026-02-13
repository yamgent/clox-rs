use crate::value::ValueArray;

#[repr(u8)]
pub enum OpCode {
    Return = 0,
    Constant = 1,
    // remember to modify the following areas when adding
    // a new enum variant:
    //      - OpCode::try_from()
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::Return),
            1 => Ok(OpCode::Constant),
            _ => Err(()),
        }
    }
}

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
