pub struct Token<'a> {
    pub token_type:TokenType,
    pub lexeme:&'a str,
    pub line:usize
}

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
    Eof
}

impl<'a> Token<'a> {
    pub fn new(token_type:TokenType,lexeme:&'a str,line:usize)->Token {
        Token {
            token_type:token_type,
            lexeme:lexeme,
            line:line,
        }
    }
}