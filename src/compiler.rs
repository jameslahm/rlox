use core::{panic};
use std::{ops::Add};
use num::FromPrimitive;
use num_derive::FromPrimitive;    

use crate::{chunk::{Chunk, Value}, error, scanner::Scanner, token::{Token, TokenType}};

#[derive(Debug,PartialEq,PartialOrd,FromPrimitive)]
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Add<i32> for Precedence {
    type Output = Precedence;
    fn add(self, rhs: i32) -> Self::Output {
        if self == Precedence::Primary {
            return self;
        }
        FromPrimitive::from_i32((self as i32) + rhs).unwrap()
    }
}

impl From<TokenType> for Precedence {
    fn from(token_type:TokenType) -> Self {
        match token_type {
            TokenType::Minus | TokenType::Plus => Precedence::Term,
            TokenType::Slash | TokenType::Star => Precedence::Factor,

            _ => Precedence::None
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    TokenError,
    ConsumeError(String)
}

pub struct Compiler {
    pub previous: Token,
    pub current: Token,
    pub scanner: Scanner,
    pub panic_mode:bool,
    pub errors:Vec<ParseError>,
    pub chunk:Chunk
}

impl Compiler {
    pub fn new(source:String) -> Self {
        Compiler {
            previous: Token::default(),
            current: Token::default(),
            panic_mode:false,
            scanner:Scanner::new(source),
            errors:vec![],
            chunk:Chunk::new()
        }
    }

    pub fn compile(&mut self) {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, error::EXPECT_EOF);
    }

    pub fn advance(&mut self) {
        self.previous = self.current.clone();
        loop {
            self.current = self.scanner.scan();
            if self.current.token_type != TokenType::Error {
                break;
            }
            self.show_error(self.current.clone(),"Invalid Token");
            self.errors.push(ParseError::TokenError);
        }
    }

    pub fn show_error(&mut self, token: Token,message:&str) {
        if self.panic_mode{
            return;
        }
        self.panic_mode = true;
        print!("[lint {}] Error: ", token.line);

        match token.token_type {
            TokenType::Eof => print!("At end "),
            _ => print!("{} ", token.lexeme),
        }
        println!("{}",message);
    }

    pub fn consume(&mut self,token_type:TokenType,message:&str){
        if self.current.token_type==token_type {
            self.advance();
            return;
        }
        self.show_error(self.current.clone(),message);
        self.errors.push(ParseError::ConsumeError(message.to_owned()))
    }

    pub fn parse_number(&mut self){
        let v:f64 = self.previous.lexeme.parse().unwrap_or(0.0);
        let value = Value::Double(v);
        self.chunk.add_op_constant(value, self.previous.line);
    }

    pub fn parse_group(&mut self){
        self.expression();
        self.consume(TokenType::RightParen, error::EXPECT_RIGHT_PAREN_AFTER_EXPRESSION);    
    }

    pub fn parse_unary(&mut self){
        let token:Token = self.previous.clone();
        self.parse_precedence(Precedence::Unary);

        match token.token_type {
            TokenType::Minus => {
                self.chunk.add_op_negate(token.line);
            },
            TokenType::Bang => {
                self.chunk.add_op_not(token.line);
            }
            _ => {}
        }

    }

    pub fn parse_binary(&mut self){
        let token:Token = self.previous.clone();

        let precedence:Precedence = token.token_type.into();
        self.parse_precedence(precedence);
        match token.token_type {
            TokenType::Plus => self.chunk.add_op_add(token.line),
            TokenType::Minus =>self.chunk.add_op_subtract(token.line),
            TokenType::Star => self.chunk.add_op_multily(token.line),
            TokenType::Slash => self.chunk.add_op_divide(token.line),
            _ => {}
        }
    }

    pub fn parse_literal(&mut self){
        let token = self.previous.clone();
        match token.token_type {
            TokenType::False => self.chunk.add_op_false(token.line),
            TokenType::True => self.chunk.add_op_true(token.line),
            TokenType::Nil => self.chunk.add_op_nil(token.line),
            _ => {}
        }
    }

    pub fn expression(&mut self){
        self.parse_precedence(Precedence::Assignment);
    }

    pub fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        self.parse_prefix();

        while precedence <= Precedence::from(self.current.token_type) {
            self.advance();
            self.parse_infix();
        }
    }

    pub fn parse_prefix(&mut self) {
        let token = self.previous.clone();
        match token.token_type {
            TokenType::LeftParen => self.parse_group(),
            TokenType::Minus => self.parse_unary(),
            TokenType::Number => self.parse_number(),
            TokenType::True | TokenType::False | TokenType::Nil =>self.parse_literal(),
            _ => {panic!("Error prefix parse")}
        }
    }

    pub fn parse_infix(&mut self) {
        let token= self.previous.clone();
        match token.token_type {
            TokenType::Minus | TokenType::Plus | TokenType::Star | TokenType::Slash => self.parse_binary(),
            _ => {panic!("Error infix parse")}
        }
    }
}
