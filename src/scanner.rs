use crate::token::{self, Token, TokenType};
use crete::util;

pub struct Scanner<'a> {
    pub source: &'a String,
    pub current: usize,
    pub start: usize,
    pub line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String) -> Scanner {
        Scanner {
            source: source,
            current: 0,
            start: 0,
            line: 0,
        }
    }

    pub fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                b'\r' | b' ' | b'\t' => {
                    self.advance();
                    continue;
                }
                b'\n' => {
                    self.line += 1;
                    self.advance();
                    continue;
                }
                b'/' => {
                    if self.peek_next() == b'/' {
                        while !self.is_at_end() && self.peek() != b'\n' {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    pub fn peek_next(&self) -> u8 {
        let index = self.current + 1;
        if index < self.source.len() {
            self.source.as_bytes()[index]
        } else {
            b'\0'
        }
    }

    pub fn scan(&mut self) -> Token {
        self.skip_whitespace();

        self.start = self.current;
        if self.is_at_end() {
            return self.token(TokenType::Eof);
        }

        let c = self.advance();

        if util::is_alpha(c) {
            return self.identifier_token();
        }

        match c {
            b'(' => self.token(TokenType::LeftParen),
            b')' => self.token(TokenType::RightParen),
            b'{' => self.token(TokenType::LeftBrace),
            b'}' => self.token(TokenType::RightBrace),
            b';' => self.token(TokenType::SemiColon),
            b',' => self.token(TokenType::Comma),
            b'.' => self.token(TokenType::Dot),
            b'-' => self.token(TokenType::Minus),
            b'+' => self.token(TokenType::Plus),
            b'/' => self.token(TokenType::Slash),
            b'*' => self.token(TokenType::Star),
            b'!' => {
                let token_type = if self.match_byte(b'=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.token(token_type)
            }
            b'=' => {
                let token_type = if self.match_byte(b'=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.token(token_type)
            }
            b'<' => {
                let token_type = if self.match_byte(b'=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.token(token_type)
            }
            b'>' => {
                let token_type = if self.match_byte(b'=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.token(token_type)
            }
            b'"' => self.string_token(),
            b'1'..=b'9' => self.number_token(),
            _ => self.token(TokenType::Error),
        }
    }

    pub fn identifier_token(&mut self)->Token {
        while (util::is_alpha(self.peek()) || util::is_digit(self.peek())) && !self.is_at_end() {
            self.advance();
        }
        match &self.source[self.start..self.current] {
            "and" => self.token(TokenType::And),
            "class" => self.token(TokenType::Class),
            "else"=>self.token(TokenType::Else),
            "if"=>self.token(TokenType::If),
            "nil"=>self.token(TokenType::Nil),
            "or"=>self.token(TokenType::Or),
            "print"=>self.token(TokenType::Print),
            "return"=>self.token(TokenType::Return),
            "super"=>self.token(TokenType::Super),
            "var"=>self.token(TokenType::Var),
            "while"=>self.token(TokenType::While),
            "false"=>self.token(TokenType::False),
            "for"=>self.token(TokenType::For),
            "fun"=>self.token(TokenType::Fun),
            "this"=>self.token(TokenType::This),
            "true"=>self.token(TokenType::True),
            _ => self.token(TokenType::Identifier)
        }
    }

    pub fn number_token(&mut self) -> Token {
        while util::is_digit(self.peek()) && !self.is_at_end() {
            self.advance();
        }
        if self.peek() == b'.' && util::is_digit(self.peek_next()) {
            self.advance();
            while util::is_digit(self.peek()) && !self.is_at_end() {
                self.advance();
            }
        }
        self.token(TokenType::Number)
    }

    pub fn string_token(&mut self) -> Token {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.token(TokenType::Error);
        }

        self.advance();
        self.token(TokenType::String)
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn advance(&mut self) -> u8 {
        self.current += 1;
        self.source.as_bytes()[self.current - 1]
    }

    pub fn token(&self, token_type: TokenType) -> Token {
        match token_type {
            TokenType::Eof => Token::new(token_type, "", self.line),
            TokenType::Error => Token::new(token_type, "Unexpected character", self.line),
            TokenType::String => Token::new(
                token_type,
                &self.source[self.start + 1..self.current - 1],
                self.line,
            ),
            TokenType::Number => {
                Token::new(token_type, &self.source[self.start..self.current], self.line)
            }
            _ => Token::new(
                token_type,
                &self.source[self.start..self.current],
                self.line,
            ),
        }
    }

    pub fn match_byte(&mut self, c: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.peek() == c {
            self.current += 1;
            return true;
        }
        false
    }

    pub fn peek(&self) -> u8 {
        self.source.as_bytes()[self.current]
    }
}
