pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
}

#[derive(Debug, PartialEq, Eq)]
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
    EndOfFile,
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
            return self.make_token(TokenKind::EndOfFile);
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
        if self.is_at_end() || self.source.as_bytes()[self.current] as char != expected {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan() {
        {
            let mut scanner = Scanner::new("(){},.-+;/*".to_string());
            assert_eq!(scanner.scan_token().kind, TokenKind::LeftParen);
            assert_eq!(scanner.scan_token().kind, TokenKind::RightParen);
            assert_eq!(scanner.scan_token().kind, TokenKind::LeftBrace);
            assert_eq!(scanner.scan_token().kind, TokenKind::RightBrace);
            assert_eq!(scanner.scan_token().kind, TokenKind::Comma);
            assert_eq!(scanner.scan_token().kind, TokenKind::Dot);
            assert_eq!(scanner.scan_token().kind, TokenKind::Minus);
            assert_eq!(scanner.scan_token().kind, TokenKind::Plus);
            assert_eq!(scanner.scan_token().kind, TokenKind::Semicolon);
            assert_eq!(scanner.scan_token().kind, TokenKind::Slash);
            assert_eq!(scanner.scan_token().kind, TokenKind::Star);
            assert_eq!(scanner.scan_token().kind, TokenKind::EndOfFile);
        }

        {
            let mut scanner = Scanner::new("! != = == > >= < <=".to_string());
            assert_eq!(scanner.scan_token().kind, TokenKind::Bang);
            assert_eq!(scanner.scan_token().kind, TokenKind::BangEqual);
            assert_eq!(scanner.scan_token().kind, TokenKind::Equal);
            assert_eq!(scanner.scan_token().kind, TokenKind::EqualEqual);
            assert_eq!(scanner.scan_token().kind, TokenKind::Greater);
            assert_eq!(scanner.scan_token().kind, TokenKind::GreaterEqual);
            assert_eq!(scanner.scan_token().kind, TokenKind::Less);
            assert_eq!(scanner.scan_token().kind, TokenKind::LessEqual);
        }

        {
            let mut scanner = Scanner::new("abc".to_string());
            let token = scanner.scan_token();
            assert_eq!(token.kind, TokenKind::Identifier);
            assert_eq!(token.lexeme, "abc");
        }

        {
            let mut scanner = Scanner::new(r#""Quick brown fox\n over lazy dog""#.to_string());
            let token = scanner.scan_token();
            assert_eq!(token.kind, TokenKind::String);
            assert_eq!(token.lexeme, r#""Quick brown fox\n over lazy dog""#);
        }

        {
            let mut scanner = Scanner::new("1.3".to_string());
            let token = scanner.scan_token();
            assert_eq!(token.kind, TokenKind::Number);
            assert_eq!(token.lexeme, "1.3");
        }

        {
            let mut scanner = Scanner::new(
                "and class else false for fun if nil or print return super this true var while"
                    .to_string(),
            );
            assert_eq!(scanner.scan_token().kind, TokenKind::And);
            assert_eq!(scanner.scan_token().kind, TokenKind::Class);
            assert_eq!(scanner.scan_token().kind, TokenKind::Else);
            assert_eq!(scanner.scan_token().kind, TokenKind::False);
            assert_eq!(scanner.scan_token().kind, TokenKind::For);
            assert_eq!(scanner.scan_token().kind, TokenKind::Fun);
            assert_eq!(scanner.scan_token().kind, TokenKind::If);
            assert_eq!(scanner.scan_token().kind, TokenKind::Nil);
            assert_eq!(scanner.scan_token().kind, TokenKind::Or);
            assert_eq!(scanner.scan_token().kind, TokenKind::Print);
            assert_eq!(scanner.scan_token().kind, TokenKind::Return);
            assert_eq!(scanner.scan_token().kind, TokenKind::Super);
            assert_eq!(scanner.scan_token().kind, TokenKind::This);
            assert_eq!(scanner.scan_token().kind, TokenKind::True);
            assert_eq!(scanner.scan_token().kind, TokenKind::Var);
            assert_eq!(scanner.scan_token().kind, TokenKind::While);
            assert_eq!(scanner.scan_token().kind, TokenKind::EndOfFile);
        }

        {
            let mut scanner = Scanner::new("~".to_string());
            let token = scanner.scan_token();
            assert_eq!(token.kind, TokenKind::Error);
        }
    }

    #[test]
    fn test_whitespace() {
        {
            let mut scanner = Scanner::new(
                r#"
// this should be ignored
fun hi() {
    // return!
    return; 
}
"#
                .to_string(),
            );
            assert_eq!(scanner.scan_token().kind, TokenKind::Fun);
            assert_eq!(scanner.scan_token().kind, TokenKind::Identifier);
            assert_eq!(scanner.scan_token().kind, TokenKind::LeftParen);
            assert_eq!(scanner.scan_token().kind, TokenKind::RightParen);
            assert_eq!(scanner.scan_token().kind, TokenKind::LeftBrace);
            assert_eq!(scanner.scan_token().kind, TokenKind::Return);
            assert_eq!(scanner.scan_token().kind, TokenKind::Semicolon);
            assert_eq!(scanner.scan_token().kind, TokenKind::RightBrace);
            assert_eq!(scanner.scan_token().kind, TokenKind::EndOfFile);
        }
    }

    #[test]
    fn test_line() {
        let mut scanner = Scanner::new(
            r#"var
and or
this
;
"#
            .to_string(),
        );
        assert_eq!(scanner.scan_token().line, 1); // var
        assert_eq!(scanner.scan_token().line, 2); // and
        assert_eq!(scanner.scan_token().line, 2); // or
        assert_eq!(scanner.scan_token().line, 3); // this
        assert_eq!(scanner.scan_token().line, 4); // ;
        assert_eq!(scanner.scan_token().line, 5); // EOF
    }
}
