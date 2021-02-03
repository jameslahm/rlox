#[derive(Debug,Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: i32,
}

#[derive(Debug,Clone, Copy,PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
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
    Equal,
    EqualEqual,

    Error,
    Eof,
}

impl<'a> Token {
    pub fn new(token_type: TokenType, lexeme: &'a str, line: i32) -> Token {
        Token {
            token_type: token_type,
            lexeme: lexeme.to_owned(),
            line: line,
        }
    }
    pub fn default() -> Token {
        Token {
            token_type: TokenType::Error,
            lexeme: String::from(""),
            line: 0,
        }
    }
}
