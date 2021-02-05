use std::rc::Rc;
use core::{panic};
use std::{ops::Add};
use num::FromPrimitive;
use num_derive::FromPrimitive;
use num_traits::float::FloatCore;    

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
            TokenType::BangEqual | TokenType::EqualEqual => Precedence::Equality,
            TokenType::Greater | TokenType::GreaterEqual => Precedence::Comparison,
            TokenType::Less | TokenType::LessEqual => Precedence::Comparison,
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
        while !self.match_token(TokenType::Eof){
            self.parse_declaration();
        }
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
        self.parse_expression();
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
            TokenType::BangEqual => {
                self.chunk.add_op_equal(token.line);
                self.chunk.add_op_not(token.line);
            },
            TokenType::EqualEqual=> {
                self.chunk.add_op_equal(token.line);
            },
            TokenType::Greater => {
                self.chunk.add_op_greater(token.line);
            },
            TokenType::GreaterEqual=>{
                self.chunk.add_op_less(token.line);
                self.chunk.add_op_not(token.line);
            },
            TokenType::Less => {
                self.chunk.add_op_less(token.line);
            },
            TokenType::LessEqual => {
                self.chunk.add_op_greater(token.line);
                self.chunk.add_op_not(token.line);
            }
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

    pub fn parse_expression(&mut self){
        self.parse_precedence(Precedence::Assignment);
    }

    pub fn parse_string(&mut self){
        let token = self.previous.clone();
        self.chunk.add_op_constant(Value::String(Rc::new(token.lexeme)),token.line);
    }

    pub fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        self.parse_prefix();

        while precedence <= Precedence::from(self.current.token_type) {
            self.advance();
            self.parse_infix();
        }
    }

    pub fn parse_statement(&mut self){
        match self.current.token_type {
            TokenType::Print => {
                self.advance();
                self.parse_print_statement();
            },
            _ => self.parse_expression_statement()
        }
    }

    pub fn parse_expression_statement(&mut self){
        self.parse_expression();
        self.consume(TokenType::SemiColon,error::EXPECT_SEMICOLON_AFTER_EXPRESSION);
        
    }

    pub fn parse_print_statement(&mut self){
        self.parse_expression();
        self.consume(TokenType::SemiColon,error::EXPECT_SEMICOLON_AFTER_VALUE);

        let token = self.previous.clone();
        self.chunk.add_op_print(token.line);
    }

    pub fn parse_var_declaration(&mut self){
        self.consume(TokenType::Identifier, error::EXPECT_VARIABLE_NAME);
        let token = self.previous.clone();
        let index= self.chunk.add_value(Value::String(Rc::new(token.lexeme)));
        if self.match_token(TokenType::Equal){
            self.parse_expression();
        } else {
            self.chunk.add_op_nil(token.line);
        }

        self.consume(TokenType::SemiColon, error::EXPECT_SEMICOLON_AFTER_VARIABLE_DECLARATION);
        self.chunk.add_op_define_global(index, token.line);
    }

    pub fn parse_variable(&mut self){
        let token = self.previous.clone();
        let index = self.chunk.add_value(Value::String(Rc::new(token.lexeme)));
        self.chunk.add_op_get_global(index, token.line);
    }

    pub fn parse_declaration(&mut self){
        match self.current.token_type {
            TokenType::Var => {
                self.advance();
                self.parse_var_declaration();
            }
            _ => self.parse_statement()
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    pub fn synchronize(&mut self){
        self.panic_mode = false;

        loop {
            match self.current.token_type {
                TokenType::SemiColon => {
                    self.advance();
                    break;
                },
                TokenType::Class | TokenType::Fun |
                TokenType::Var | TokenType::For |
                TokenType::If | TokenType::While |
                TokenType::Print | TokenType::Return => break,
                _ => {}
            }
            self.advance();
        }
    }

    pub fn parse_prefix(&mut self) {
        let token = self.previous.clone();
        match token.token_type {
            TokenType::LeftParen => self.parse_group(),
            TokenType::Minus => self.parse_unary(),
            TokenType::Number => self.parse_number(),
            TokenType::True | TokenType::False | TokenType::Nil =>self.parse_literal(),
            TokenType::String => self.parse_string(),
            _ => {
                self.show_error(token, error::EXPECT_EXPRESSION);
            }
        }
    }

    pub fn parse_infix(&mut self) {
        let token= self.previous.clone();
        match token.token_type {
            TokenType::Minus | TokenType::Plus | TokenType::Star | TokenType::Slash |
            TokenType::EqualEqual | TokenType::Greater | TokenType::GreaterEqual|
            TokenType::Less | TokenType::LessEqual => self.parse_binary(),
            _ => {panic!("Error infix parse")}
        }
    }

    pub fn match_token(&mut self,token_type:TokenType) -> bool{
        if !self.check(token_type){
            false
        } else {
            self.advance();
            true
        }
    }

    pub fn check(&mut self,token_type:TokenType)->bool{
        self.current.token_type == token_type
    }
}
