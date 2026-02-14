pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
}

#[derive(Debug)]
pub enum TokenKind {
    // single character token
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // one or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // literals
    Identifier,
    String,
    Number,

    // keywords
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    EOF,
}

pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token<'_> {
        self.skip_whitespace();

        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenKind::EOF);
        }

        let c = self.advance();

        if c.is_ascii_alphabetic() || c == '_' {
            return self.identifier();
        }
        if c.is_ascii_digit() {
            return self.number();
        }

        match c {
            '(' => self.make_token(TokenKind::LeftParen),
            ')' => self.make_token(TokenKind::RightParen),
            '{' => self.make_token(TokenKind::LeftBrace),
            '}' => self.make_token(TokenKind::RightBrace),
            ';' => self.make_token(TokenKind::Semicolon),
            ',' => self.make_token(TokenKind::Comma),
            '.' => self.make_token(TokenKind::Dot),
            '-' => self.make_token(TokenKind::Minus),
            '+' => self.make_token(TokenKind::Plus),
            '/' => self.make_token(TokenKind::Slash),
            '*' => self.make_token(TokenKind::Star),
            '!' => {
                let kind = if self.match_ch('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                };
                self.make_token(kind)
            }
            '=' => {
                let kind = if self.match_ch('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                };
                self.make_token(kind)
            }
            '<' => {
                let kind = if self.match_ch('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                self.make_token(kind)
            }
            '>' => {
                let kind = if self.match_ch('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                self.make_token(kind)
            }
            '"' => self.string(),
            _ => self.error_token("Unexpected character."),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.as_bytes()[self.current - 1] as char
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.as_bytes()[self.current] as char
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.as_bytes()[self.current + 1] as char
        }
    }

    fn match_ch(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            false
        } else if self.source.as_bytes()[self.current] as char != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn make_token(&self, kind: TokenKind) -> Token<'_> {
        Token {
            kind,
            lexeme: &self.source[self.start..self.current],
            line: self.line,
        }
    }

    fn error_token<'a>(&self, message: &'a str) -> Token<'a> {
        Token {
            kind: TokenKind::Error,
            lexeme: message,
            line: self.line,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let ch = self.peek();

            match ch {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        // a comment goes until the end of the line
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => {
                    return;
                }
            }
        }
    }

    fn identifier(&mut self) -> Token<'_> {
        loop {
            let ch = self.peek();
            if ch.is_ascii_alphabetic() || ch.is_ascii_digit() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let identifier_type = self.identifier_type();
        self.make_token(identifier_type)
    }

    fn identifier_type(&self) -> TokenKind {
        // this is a simple "trie". The book also says that V8 actually does this as well.
        match self.source.as_bytes()[self.start] as char {
            'a' => self.check_keyword(1, "nd", TokenKind::And),
            'c' => self.check_keyword(1, "lass", TokenKind::Class),
            'e' => self.check_keyword(1, "lse", TokenKind::Else),
            'i' => self.check_keyword(1, "f", TokenKind::If),
            'n' => self.check_keyword(1, "il", TokenKind::Nil),
            'o' => self.check_keyword(1, "r", TokenKind::Or),
            'p' => self.check_keyword(1, "rint", TokenKind::Print),
            'r' => self.check_keyword(1, "eturn", TokenKind::Return),
            's' => self.check_keyword(1, "uper", TokenKind::Super),
            'v' => self.check_keyword(1, "ar", TokenKind::Var),
            'w' => self.check_keyword(1, "hile", TokenKind::While),
            'f' => {
                if self.current - self.start > 1 {
                    match self.source.as_bytes()[self.start + 1] as char {
                        'a' => self.check_keyword(2, "lse", TokenKind::False),
                        'o' => self.check_keyword(2, "r", TokenKind::For),
                        'u' => self.check_keyword(2, "n", TokenKind::Fun),
                        _ => TokenKind::Identifier,
                    }
                } else {
                    TokenKind::Identifier
                }
            }
            't' => {
                if self.current - self.start > 1 {
                    match self.source.as_bytes()[self.start + 1] as char {
                        'h' => self.check_keyword(2, "is", TokenKind::This),
                        'r' => self.check_keyword(2, "ue", TokenKind::True),
                        _ => TokenKind::Identifier,
                    }
                } else {
                    TokenKind::Identifier
                }
            }
            _ => TokenKind::Identifier,
        }
    }

    fn check_keyword(&self, start: usize, rest: &str, kind: TokenKind) -> TokenKind {
        if self.current - self.start == start + rest.len()
            && &self.source[(self.start + start)..(self.start + start + rest.len())] == rest
        {
            kind
        } else {
            TokenKind::Identifier
        }
    }

    fn number(&mut self) -> Token<'_> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // look for a fractional part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // consume the "."
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.make_token(TokenKind::Number)
    }

    fn string(&mut self) -> Token<'_> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error_token("Unterminated string.")
        } else {
            self.advance();
            self.make_token(TokenKind::String)
        }
    }
}
