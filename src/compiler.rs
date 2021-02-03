use std::ops::Add;
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

    pub fn compile(&self) {
    
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
        let value:Value = self.previous.lexeme.parse().unwrap_or(0.0);
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
            }
            _ => {}
        }

    }

    pub fn parse_binary(&mut self){
        let token:Token = self.previous.clone();


    }

    pub fn expression(&mut self){
        self.parse_precedence(Precedence::Assignment);
    }

    pub fn parse_precedence(&self, precedence: Precedence) {}
}
