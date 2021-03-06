use crate::token::{Token, TokenType};

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 0,
        }
    }

    fn is_digit(c: char) -> bool {
        match c {
            '0'..='9' => true,
            _ => false,
        }
    }

    fn is_alpha(c: char) -> bool {
        match c {
            'a'..='z' | 'A'..='Z' | '_' => true,
            _ => false,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token<'a>>, String> {
        let mut tokens = Vec::new();
        loop {
            match self.scan_token() {
                Ok(tok) => {
                    tokens.push(tok);
                    if tok.typ == TokenType::Eof {
                        return Ok(tokens);
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn scan_token(&mut self) -> Result<Token<'a>, String> {
        self.skip_whitespace();

        self.start = self.current;

        if self.is_at_end() {
            return Ok(self.make_token(TokenType::Eof));
        }

        let c = self.advance();

        if Self::is_digit(c) {
            return Ok(self.number());
        }

        if Self::is_alpha(c) {
            return Ok(self.identifier());
        }

        match c {
            '(' => Ok(self.make_token(TokenType::LeftParen)),
            ')' => Ok(self.make_token(TokenType::RightParen)),
            '{' => Ok(self.make_token(TokenType::LeftBrace)),
            '}' => Ok(self.make_token(TokenType::RightBrace)),
            ';' => Ok(self.make_token(TokenType::SemiColon)),
            ',' => Ok(self.make_token(TokenType::Comma)),
            '.' => Ok(self.make_token(TokenType::Dot)),
            '-' => Ok(self.make_token(TokenType::Minus)),
            '+' => Ok(self.make_token(TokenType::Plus)),
            '/' => Ok(self.make_token(TokenType::Slash)),
            '*' => Ok(self.make_token(TokenType::Star)),
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    return Ok(self.make_token(TokenType::BangEqual));
                }
                Ok(self.make_token(TokenType::Bang))
            }
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    return Ok(self.make_token(TokenType::EqualEqual));
                }
                Ok(self.make_token(TokenType::Equal))
            }
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    return Ok(self.make_token(TokenType::GreaterEqual));
                }
                Ok(self.make_token(TokenType::Greater))
            }
            '<' => {
                if self.peek() == '=' {
                    self.advance();
                    return Ok(self.make_token(TokenType::LessEqual));
                }
                Ok(self.make_token(TokenType::Less))
            }
            '"' => self.string(),
            _ => Err(format!("Unexpected character at {}", self.line)),
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current);
        self.current = self.current + 1;
        c.expect("Scanner tried to advance to out of bounds character")
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source
            .chars()
            .nth(self.current)
            .expect("Scanner tried to advance to out of bounds character")
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source
            .chars()
            .nth(self.current + 1)
            .expect("Scanner tried to advance to out of bounds character")
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line = self.line + 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        // A comment goes until the end of the line.
                        while !self.is_at_end() && self.peek() != '\n' {
                            self.advance();
                        }
                    }
                }
                _ => return,
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn string(&mut self) -> Result<Token<'a>, String> {
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line = self.line + 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(format!("Unterminated string at {}", self.line));
        }

        // closing quote.
        self.advance();

        Ok(self.make_token(TokenType::String))
    }

    fn number(&mut self) -> Token<'a> {
        while Self::is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && Self::is_digit(self.peek_next()) {
            // look for a fractional part.
            self.advance();
            while Self::is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token<'a> {
        while Self::is_alpha(self.peek()) || Self::is_digit(self.peek()) {
            self.advance();
        }

        let typ = self.identifier_type();

        self.make_token(typ)
    }

    fn identifier_type(&self) -> TokenType {
        let c = self
            .source
            .chars()
            .nth(self.start)
            .expect("Scanner tried to peek identifier out of bounds character");

        match c {
            'a' => {
                if self.check_rest_keyword(1, "nd") {
                    TokenType::And
                } else {
                    TokenType::Identifier
                }
            }
            'c' => {
                if self.check_rest_keyword(1, "lass") {
                    TokenType::Class
                } else {
                    TokenType::Identifier
                }
            }
            'e' => {
                if self.check_rest_keyword(1, "lse") {
                    TokenType::Else
                } else {
                    TokenType::Identifier
                }
            }
            'f' => {
                if self.current - self.start >= 2 {
                    match self
                        .source
                        .chars()
                        .nth(self.start + 1)
                        .expect("Scanner tried to peek identifier out of bounds character")
                    {
                        'a' => {
                            if self.check_rest_keyword(2, "lse") {
                                TokenType::False
                            } else {
                                TokenType::Identifier
                            }
                        }
                        'o' => {
                            if self.check_rest_keyword(2, "r") {
                                TokenType::For
                            } else {
                                TokenType::Identifier
                            }
                        }
                        'u' => {
                            if self.check_rest_keyword(2, "n") {
                                TokenType::Fun
                            } else {
                                TokenType::Identifier
                            }
                        }
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'i' => {
                if self.check_rest_keyword(1, "f") {
                    TokenType::If
                } else {
                    TokenType::Identifier
                }
            }
            'n' => {
                if self.check_rest_keyword(1, "il") {
                    TokenType::Nil
                } else {
                    TokenType::Identifier
                }
            }
            'o' => {
                if self.check_rest_keyword(1, "r") {
                    TokenType::Or
                } else {
                    TokenType::Identifier
                }
            }
            'p' => {
                if self.check_rest_keyword(1, "rint") {
                    TokenType::Print
                } else {
                    TokenType::Identifier
                }
            }
            'r' => {
                if self.check_rest_keyword(1, "eturn") {
                    TokenType::Return
                } else {
                    TokenType::Identifier
                }
            }
            't' => {
                if self.current - self.start >= 2 {
                    match self
                        .source
                        .chars()
                        .nth(self.start + 1)
                        .expect("Scanner tried to peek identifier out of bounds character")
                    {
                        'h' => {
                            if self.check_rest_keyword(2, "is") {
                                TokenType::This
                            } else {
                                TokenType::Identifier
                            }
                        }
                        'r' => {
                            if self.check_rest_keyword(2, "ue") {
                                TokenType::True
                            } else {
                                TokenType::Identifier
                            }
                        }
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'v' => {
                if self.check_rest_keyword(1, "ar") {
                    TokenType::Var
                } else {
                    TokenType::Identifier
                }
            }
            'w' => {
                if self.check_rest_keyword(1, "hile") {
                    TokenType::While
                } else {
                    TokenType::Identifier
                }
            }
            _ => TokenType::Identifier,
        }
    }

    fn check_rest_keyword(&self, offset: usize, rest: &str) -> bool {
        // check length
        if self.current - (self.start + offset) != rest.len() {
            return false;
        }

        return &self.source[self.start + offset..self.current] == rest;
    }

    fn make_token(&self, typ: TokenType) -> Token<'a> {
        Token {
            typ,
            line: self.line,
            source: &self.source[self.start..self.current],
        }
    }
}
