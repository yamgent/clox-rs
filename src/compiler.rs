use std::io;

use crate::{
    chunk::{Chunk, OpCode},
    debug,
    scanner::{Scanner, Token, TokenKind},
    value::Value,
};

struct Parser {
    previous: Token,
    current: Token,
    // whether the error has appeared at any time in the compilation
    had_error: bool,
    // once the parser encounters an error, panic mode is enabled and error
    // recovery is attempted. Once error recovery is done, this is set back
    // to false. Hence, this boolean cannot tell whether an error happened in the
    // code at all. For that, use `had_error` instead.
    panic_mode: bool,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    fn plus_one(&self) -> Precedence {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => {
                // nothing higher than Primary
                Precedence::Primary
            }
        }
    }
}

pub struct Compiler {
    scanner: Scanner,
    parser: Parser,
}

impl Compiler {
    pub fn new(source: String) -> Self {
        Self {
            scanner: Scanner::new(source),
            parser: Parser {
                previous: Token {
                    kind: TokenKind::Error,
                    lexeme: "Nothing is read yet.".to_string(),
                    line: 0,
                },
                current: Token {
                    kind: TokenKind::Error,
                    lexeme: "Nothing is read yet.".to_string(),
                    line: 0,
                },
                had_error: false,
                panic_mode: false,
            },
        }
    }

    pub fn compile(&mut self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();

        self.advance();
        self.expression(&mut chunk);
        self.consume(TokenKind::EndOfFile, "Expect end of expression.");
        self.end_compiler(&mut chunk);

        if self.parser.had_error {
            Err(())
        } else {
            Ok(chunk)
        }
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current.clone();

        loop {
            self.parser.current = self.scanner.scan_token();
            if matches!(self.parser.current.kind, TokenKind::Error) {
                let message = self.parser.current.lexeme.clone();
                self.error_at_current(message);
            } else {
                break;
            }
        }
    }

    fn consume<S: AsRef<str>>(&mut self, token_kind: TokenKind, message: S) {
        if self.parser.current.kind == token_kind {
            self.advance();
        } else {
            self.error_at_current(message);
        }
    }

    fn emit_byte(&self, chunk: &mut Chunk, byte: u8) {
        chunk.write(byte, self.parser.previous.line as u32);
    }

    fn emit_bytes(&self, chunk: &mut Chunk, bytes: &[u8]) {
        bytes.iter().for_each(|byte| self.emit_byte(chunk, *byte));
    }

    fn end_compiler(&self, chunk: &mut Chunk) {
        self.emit_return(chunk);

        if debug::is_debug_print_code_enabled() && !self.parser.had_error {
            debug::disassemble_chunk(&mut io::stdout(), chunk, "code");
        }
    }

    fn binary(&mut self, chunk: &mut Chunk) {
        let operator_type = self.parser.previous.kind;
        self.parse_precedence(chunk, self.get_rule_precedence(operator_type).plus_one());

        match operator_type {
            TokenKind::Plus => {
                self.emit_byte(chunk, OpCode::Add as u8);
            }
            TokenKind::Minus => {
                self.emit_byte(chunk, OpCode::Subtract as u8);
            }
            TokenKind::Star => {
                self.emit_byte(chunk, OpCode::Multiply as u8);
            }
            TokenKind::Slash => {
                self.emit_byte(chunk, OpCode::Divide as u8);
            }
            TokenKind::BangEqual => {
                self.emit_bytes(chunk, &[OpCode::Equal as u8, OpCode::Not as u8]);
            }
            TokenKind::EqualEqual => {
                self.emit_byte(chunk, OpCode::Equal as u8);
            }
            TokenKind::Greater => {
                self.emit_byte(chunk, OpCode::Greater as u8);
            }
            // this desugaring means that "NaN >= 1" will be true, violating IEEE-754 where it
            // should be false. this is done intentionally by the book to make implementation
            // simpler
            TokenKind::GreaterEqual => {
                self.emit_bytes(chunk, &[OpCode::Less as u8, OpCode::Not as u8]);
            }
            TokenKind::Less => {
                self.emit_byte(chunk, OpCode::Less as u8);
            }
            // this desugaring means that "NaN <= 1" will be true, violating IEEE-754 where it
            // should be false. this is done intentionally by the book to make implementation
            // simpler
            TokenKind::LessEqual => {
                self.emit_bytes(chunk, &[OpCode::Greater as u8, OpCode::Not as u8]);
            }
            _ => {
                panic!("ICE: Unhandled binary");
            }
        }
    }

    fn literal(&mut self, chunk: &mut Chunk) {
        let operator_type = self.parser.previous.kind;

        match operator_type {
            TokenKind::False => {
                self.emit_byte(chunk, OpCode::False as u8);
            }
            TokenKind::True => {
                self.emit_byte(chunk, OpCode::True as u8);
            }
            TokenKind::Nil => {
                self.emit_byte(chunk, OpCode::Nil as u8);
            }
            _ => {
                panic!("ICE: Unhandled literal");
            }
        }
    }

    fn grouping(&mut self, chunk: &mut Chunk) {
        self.expression(chunk);
        self.consume(TokenKind::RightParen, "Expect ')' after expression.");
    }

    fn number(&self, chunk: &mut Chunk) {
        let value = self
            .parser
            .previous
            .lexeme
            .parse::<f64>()
            .expect("ICE: Non-number stored in number token?");
        self.emit_constant(chunk, Value::Number(value));
    }

    fn unary(&mut self, chunk: &mut Chunk) {
        let operator_type = self.parser.previous.kind;

        self.parse_precedence(chunk, Precedence::Unary);

        match operator_type {
            TokenKind::Minus => {
                self.emit_byte(chunk, OpCode::Negate as u8);
            }
            TokenKind::Bang => {
                self.emit_byte(chunk, OpCode::Not as u8);
            }
            _ => {
                panic!("ICE: Unhandled unary.");
            }
        }
    }

    fn emit_return(&self, chunk: &mut Chunk) {
        self.emit_byte(chunk, OpCode::Return as u8);
    }

    fn make_constant(&self, chunk: &mut Chunk, value: Value) -> u8 {
        let constant = chunk.constants_mut().add(value);
        TryInto::<u8>::try_into(constant)
            .unwrap_or_else(|_| panic!("ICE: Too many constants in one chunk."))
    }

    fn emit_constant(&self, chunk: &mut Chunk, value: Value) {
        let constant_index = self.make_constant(chunk, value);
        self.emit_bytes(chunk, &[OpCode::Constant as u8, constant_index]);
    }

    fn expression(&mut self, chunk: &mut Chunk) {
        self.parse_precedence(chunk, Precedence::Assignment);
    }

    fn parse_precedence(&mut self, chunk: &mut Chunk, precedence: Precedence) {
        self.advance();
        self.do_rule_prefix(chunk, self.parser.previous.kind);

        while precedence <= self.get_rule_precedence(self.parser.current.kind) {
            self.advance();
            self.do_rule_infix(chunk, self.parser.previous.kind);
        }
    }

    fn get_rule_precedence(&self, kind: TokenKind) -> Precedence {
        match kind {
            TokenKind::Minus | TokenKind::Plus => Precedence::Term,
            TokenKind::Slash | TokenKind::Star => Precedence::Factor,
            TokenKind::BangEqual | TokenKind::EqualEqual => Precedence::Equality,
            TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::Less
            | TokenKind::LessEqual => Precedence::Comparison,
            _ => Precedence::None,
        }
    }

    fn do_rule_prefix(&mut self, chunk: &mut Chunk, kind: TokenKind) {
        match kind {
            TokenKind::LeftParen => {
                self.grouping(chunk);
            }
            TokenKind::Minus | TokenKind::Bang => {
                self.unary(chunk);
            }
            TokenKind::Number => {
                self.number(chunk);
            }
            TokenKind::False | TokenKind::True | TokenKind::Nil => {
                self.literal(chunk);
            }
            _ => {
                self.error("Expect expression.");
            }
        }
    }

    fn do_rule_infix(&mut self, chunk: &mut Chunk, kind: TokenKind) {
        match kind {
            TokenKind::Minus
            | TokenKind::Plus
            | TokenKind::Slash
            | TokenKind::Star
            | TokenKind::BangEqual
            | TokenKind::EqualEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::Less
            | TokenKind::LessEqual => {
                self.binary(chunk);
            }
            _ => {
                self.error("Expect expression.");
            }
        }
    }

    fn error_at_current<S: AsRef<str>>(&mut self, message: S) {
        let token = self.parser.current.clone();
        self.error_at(token, message);
    }

    fn error<S: AsRef<str>>(&mut self, message: S) {
        let token = self.parser.previous.clone();
        self.error_at(token, message);
    }

    fn error_at<S: AsRef<str>>(&mut self, token: Token, message: S) {
        if self.parser.panic_mode {
            // prevent error cascade
            return;
        }

        self.parser.panic_mode = true;
        eprint!("[line {}] Error", token.line);

        match token.kind {
            TokenKind::EndOfFile => {
                eprint!(" at end");
            }
            TokenKind::Error => {
                // nothing
            }
            _ => {
                eprint!(" at '{}'", token.lexeme);
            }
        }

        eprintln!(": {}", message.as_ref());
        self.parser.had_error = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_compile() {
        // test error
        {
            let mut compiler = Compiler::new("1 +".to_string());
            assert_eq!(compiler.compile(), Err(()));
        }

        // test unary ops
        {
            let mut compiler = Compiler::new("-3".to_string());
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(3.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Negate as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        {
            let mut compiler = Compiler::new("!true".to_string());
            let mut chunk = Chunk::new();

            chunk.write(OpCode::True as u8, 1);
            chunk.write(OpCode::Not as u8, 1);
            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        // test binary ops
        {
            let mut compiler = Compiler::new("1 + 2".to_string());
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(1.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(2.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Add as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        {
            let mut compiler = Compiler::new("8 - 3".to_string());
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(8.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(3.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Subtract as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        {
            let mut compiler = Compiler::new("5 * 6".to_string());
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(5.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(6.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Multiply as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        {
            let mut compiler = Compiler::new("28 / 4".to_string());
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(28.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(4.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Divide as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        {
            let mut compiler = Compiler::new("true == nil".to_string());
            let mut chunk = Chunk::new();

            chunk.write(OpCode::True as u8, 1);

            chunk.write(OpCode::Nil as u8, 1);

            chunk.write(OpCode::Equal as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        {
            let mut compiler = Compiler::new("false != nil".to_string());
            let mut chunk = Chunk::new();

            chunk.write(OpCode::False as u8, 1);

            chunk.write(OpCode::Nil as u8, 1);

            chunk.write(OpCode::Equal as u8, 1);
            chunk.write(OpCode::Not as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        {
            let mut compiler = Compiler::new("3 > 4".to_string());
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(3.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(4.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Greater as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        {
            let mut compiler = Compiler::new("3 >= 4".to_string());
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(3.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(4.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            // this desugaring means that "NaN >= 1" will be true, violating IEEE-754 where it
            // should be false. this is done intentionally by the book to make implementation
            // simpler
            chunk.write(OpCode::Less as u8, 1);
            chunk.write(OpCode::Not as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        {
            let mut compiler = Compiler::new("3 < 4".to_string());
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(3.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(4.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Less as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        {
            let mut compiler = Compiler::new("3 <= 4".to_string());
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(3.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(4.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            // this desugaring means that "NaN <= 1" will be true, violating IEEE-754 where it
            // should be false. this is done intentionally by the book to make implementation
            // simpler
            chunk.write(OpCode::Greater as u8, 1);
            chunk.write(OpCode::Not as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        // test complex expressions
        {
            let mut compiler = Compiler::new("(-1 + 2) * 3 - -4".to_string());
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(1.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Negate as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(2.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Add as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(3.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Multiply as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(4.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Negate as u8, 1);

            chunk.write(OpCode::Subtract as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        // test multi-line
        {
            let mut compiler = Compiler::new("5\n*\n6".to_string());
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(5.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(6.0));
            chunk.write(OpCode::Constant as u8, 3);
            chunk.write(constant as u8, 3);

            // NOTE: line = 3 is deliberate, the book acknowledge this
            // flaw, and we are too lazy to come up with a solution
            chunk.write(OpCode::Multiply as u8, 3);

            chunk.write(OpCode::Return as u8, 3);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        // test basic arithmetic precedences
        {
            let mut compiler = Compiler::new("1 - 4 * 6".to_string());
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(1.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(4.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(6.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Multiply as u8, 1);

            chunk.write(OpCode::Subtract as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }

        {
            let mut compiler = Compiler::new("1 * 4 - 6".to_string());
            let mut chunk = Chunk::new();

            let constant = chunk.constants_mut().add(Value::Number(1.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(4.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Multiply as u8, 1);

            let constant = chunk.constants_mut().add(Value::Number(6.0));
            chunk.write(OpCode::Constant as u8, 1);
            chunk.write(constant as u8, 1);

            chunk.write(OpCode::Subtract as u8, 1);

            chunk.write(OpCode::Return as u8, 1);

            assert_eq!(compiler.compile(), Ok(chunk));
        }
    }
}
