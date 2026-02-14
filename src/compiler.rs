use crate::scanner::{Scanner, TokenKind};

pub struct Compiler {
    scanner: Scanner,
}

impl Compiler {
    pub fn new(source: String) -> Self {
        Self {
            scanner: Scanner::new(source),
        }
    }

    pub fn compile(&mut self) {
        let mut first_line = true;
        let mut line = 0;

        loop {
            let token = self.scanner.scan_token();

            if first_line || token.line != line {
                print!("{:4} ", token.line);
                line = token.line;
                first_line = false;
            } else {
                print!("   | ");
            }
            println!("{:?} '{}'", token.kind, token.lexeme);

            if matches!(token.kind, TokenKind::EOF) {
                break;
            }
        }
    }
}
