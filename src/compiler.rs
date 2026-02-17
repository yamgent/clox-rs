use crate::{
    chunk::Chunk,
    scanner::{Scanner, Token, TokenKind},
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
        self.advance();
        self.expression();
        self.consume(TokenKind::EndOfFile, "Expect end of expression.");

        if self.parser.had_error {
            Err(())
        } else {
            Ok(Chunk::new())
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

    fn expression(&mut self) {
        // TODO: implement
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

// TODO: Test
