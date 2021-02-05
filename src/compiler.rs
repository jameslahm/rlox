use core::{panic, panicking::panic};
use num::FromPrimitive;
use num_derive::FromPrimitive;

use std::{fs::create_dir, process::exit, rc::Rc};
use std::{ops::Add, vec};

use crate::{
    chunk::{Chunk, Value},
    error,
    scanner::Scanner,
    token::{Token, TokenType},
};

use crate::op_code::OpCode;

#[derive(Debug, PartialEq, PartialOrd, FromPrimitive, Clone, Copy)]
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
    fn from(token_type: TokenType) -> Self {
        match token_type {
            TokenType::Minus | TokenType::Plus => Precedence::Term,
            TokenType::Slash | TokenType::Star => Precedence::Factor,
            TokenType::BangEqual | TokenType::EqualEqual => Precedence::Equality,
            TokenType::Greater | TokenType::GreaterEqual => Precedence::Comparison,
            TokenType::Less | TokenType::LessEqual => Precedence::Comparison,
            _ => Precedence::None,
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    TokenError,
    ConsumeError(String),
}

pub struct Local {
    pub name: String,
    pub depth: u32,
}

pub struct Compiler {
    pub previous: Token,
    pub current: Token,
    pub scanner: Scanner,
    pub panic_mode: bool,
    pub errors: Vec<ParseError>,
    pub chunk: Chunk,
    pub scope_depth: u32,
    pub locals: Vec<Local>,
}

impl Compiler {
    pub fn new(source: String) -> Self {
        Compiler {
            previous: Token::default(),
            current: Token::default(),
            panic_mode: false,
            scanner: Scanner::new(source),
            errors: vec![],
            chunk: Chunk::new(),
            scope_depth: 0,
            locals: vec![],
        }
    }

    pub fn compile(&mut self) {
        self.advance();
        while !self.match_token(TokenType::Eof) {
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
            self.show_error(self.current.clone(), "Invalid Token");
            self.errors.push(ParseError::TokenError);
        }
    }

    pub fn show_error(&mut self, token: Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        print!("[lint {}] Error: ", token.line);

        match token.token_type {
            TokenType::Eof => print!("At end "),
            _ => print!("{} ", token.lexeme),
        }
        println!("{}", message);
    }

    pub fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.current.token_type == token_type {
            self.advance();
            return;
        }
        self.show_error(self.current.clone(), message);
        self.errors
            .push(ParseError::ConsumeError(message.to_owned()))
    }

    pub fn parse_number(&mut self) {
        let v: f64 = self.previous.lexeme.parse().unwrap_or(0.0);
        let value = Value::Double(v);
        self.chunk.add_op_constant(value, self.previous.line);
    }

    pub fn parse_group(&mut self) {
        self.parse_expression();
        self.consume(
            TokenType::RightParen,
            error::EXPECT_RIGHT_PAREN_AFTER_EXPRESSION,
        );
    }

    pub fn parse_unary(&mut self) {
        let token: Token = self.previous.clone();
        self.parse_precedence(Precedence::Unary);

        match token.token_type {
            TokenType::Minus => {
                self.chunk.add_op_negate(token.line);
            }
            TokenType::Bang => {
                self.chunk.add_op_not(token.line);
            }
            _ => {}
        }
    }

    pub fn parse_binary(&mut self) {
        let token: Token = self.previous.clone();

        let precedence: Precedence = token.token_type.into();
        self.parse_precedence(precedence);
        match token.token_type {
            TokenType::Plus => self.chunk.add_op_add(token.line),
            TokenType::Minus => self.chunk.add_op_subtract(token.line),
            TokenType::Star => self.chunk.add_op_multily(token.line),
            TokenType::Slash => self.chunk.add_op_divide(token.line),
            TokenType::BangEqual => {
                self.chunk.add_op_equal(token.line);
                self.chunk.add_op_not(token.line);
            }
            TokenType::EqualEqual => {
                self.chunk.add_op_equal(token.line);
            }
            TokenType::Greater => {
                self.chunk.add_op_greater(token.line);
            }
            TokenType::GreaterEqual => {
                self.chunk.add_op_less(token.line);
                self.chunk.add_op_not(token.line);
            }
            TokenType::Less => {
                self.chunk.add_op_less(token.line);
            }
            TokenType::LessEqual => {
                self.chunk.add_op_greater(token.line);
                self.chunk.add_op_not(token.line);
            }
            _ => {}
        }
    }

    pub fn parse_literal(&mut self) {
        let token = self.previous.clone();
        match token.token_type {
            TokenType::False => self.chunk.add_op_false(token.line),
            TokenType::True => self.chunk.add_op_true(token.line),
            TokenType::Nil => self.chunk.add_op_nil(token.line),
            _ => {}
        }
    }

    pub fn parse_expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    pub fn parse_string(&mut self) {
        let token = self.previous.clone();
        self.chunk
            .add_op_constant(Value::String(Rc::new(token.lexeme)), token.line);
    }

    pub fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        self.parse_prefix(precedence);

        while precedence <= Precedence::from(self.current.token_type) {
            self.advance();
            self.parse_infix();
        }

        if precedence <= Precedence::Assignment && self.match_token(TokenType::Equal) {
            self.show_error(self.previous.clone(), error::INVALID_ASSIGNMENT_TARGET);
        }
    }

    pub fn parse_statement(&mut self) {
        match self.current.token_type {
            TokenType::Print => {
                self.advance();
                self.parse_print_statement();
            }
            TokenType::LeftBrace => {
                self.advance();
                self.enter_scope();
                self.parse_block_statement();
                self.exit_scope();
            }
            TokenType::If => {
                self.advance();
                self.parse_if_statement();
            }
            TokenType::While => {
                self.advance();
                self.parse_while_statement();
            }
            TokenType::For => {
                self.advance();
                self.parse_for_statement();
            }
            _ => self.parse_expression_statement(),
        }
    }

    pub fn parse_for_statement(&mut self){
        self.enter_scope();
        self.consume(TokenType::LeftParen, error::EXPECT_LEFT_PAREN_AFTER_FOR);
        if self.match_token(TokenType::SemiColon) {
        } else if self.match_token(TokenType::Var){
            self.parse_var_declaration();
        } else{
            self.parse_expression_statement();
        }

        let exit_index:i32= -1;
        let condition_index = self.chunk.codes.len();
        if !self.match_token(TokenType::SemiColon) {
            self.parse_expression();
            self.consume(TokenType::SemiColon, error::EXPECT_SEMICOLON_AFTER_LOOP);

            exit_index = self.chunk.add_op_juml_if_false(0, self.previous.line) as i32;
            self.chunk.add_op_pop(self.previous.line);
        }

        let incre_index = self.chunk.codes.len();

        if !self.match_token(TokenType::RightParen){
            let body_index = self.chunk.add_op_jump(0, self.previous.line);
            self.parse_expression();
            self.chunk.add_op_pop(self.previous.line);
            self.chunk.add_op_loop(condition_index,self.previous.line);
            self.patch_op(body_index);
        }

        self.chunk.add_op_loop(incre_index,self.previous.line);

        if exit_index!=-1 {
            self.patch_op(exit_index as usize);
            self.chunk.add_op_pop(self.previous.line);
        }
        self.exit_scope();
    }
    
    pub fn parse_while_statement(&mut self) {
        let loop_index = self.chunk.codes.len();

        self.consume(TokenType::LeftParen, error::EXPECT_LEFT_PAREN_AFTER_WHILE);
        self.parse_expression();
        self.consume(
            TokenType::RightParen,
            error::EXPECT_RIGHT_PAREN_AFTER_CONDITION,
        );

        let exit_index = self.chunk.add_op_juml_if_false(0, self.previous.line);
        self.chunk.add_op_pop(self.previous.line);
        self.parse_statement();
        self.chunk.add_op_loop(self.chunk.codes.len()-loop_index, self.previous.line);

        self.patch_op(exit_index);
        self.chunk.add_op_pop(self.previous.line);
    }

    pub fn parse_if_statement(&mut self) {
        self.consume(TokenType::LeftParen, error::EXPECT_LEFT_PAREN_AFTER_IF);
        self.parse_expression();
        self.consume(
            TokenType::RightParen,
            error::EXPECT_RIGHT_PAREN_AFTER_CONDITION,
        );

        let then_index = self.chunk.add_op_juml_if_false(0, self.previous.line);
        self.chunk.add_op_pop(self.previous.line);
        self.parse_statement();

        let else_index = self.chunk.add_op_jump(0, self.previous.line);

        self.patch_op(then_index);
        self.chunk.add_op_pop(self.previous.line);

        if self.match_token(TokenType::Else) {
            self.parse_statement();
        }
        self.patch_op(else_index);
    }

    pub fn patch_op(&mut self, index: usize) {
        let mut op = &self.chunk.codes[index];
        match op {
            OpCode::OpJumpIfFalse(ref mut offset) => {
                *offset = self.chunk.codes.len() - index;
            }
            OpCode::OpJump(ref mut offset) => {
                *offset = self.chunk.codes.len() - index;
            }
            _ => {
                panic!("Path not jump")
            }
        }
    }

    pub fn parse_block_statement(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.parse_declaration();
        }
        self.consume(TokenType::RightBrace, error::EXPECT_RIGHT_BRACE_AFTER_BLOCK);
    }
    pub fn enter_scope(&mut self) {
        self.scope_depth += 1;
    }
    pub fn exit_scope(&mut self) {
        self.scope_depth -= 1;
        while self.locals[self.locals.len() - 1].depth > self.scope_depth {
            self.locals.remove(self.locals.len());
            self.chunk.add_op_pop(self.previous.line);
        }
    }

    pub fn parse_expression_statement(&mut self) {
        self.parse_expression();
        self.consume(
            TokenType::SemiColon,
            error::EXPECT_SEMICOLON_AFTER_EXPRESSION,
        );
    }

    pub fn parse_print_statement(&mut self) {
        self.parse_expression();
        self.consume(TokenType::SemiColon, error::EXPECT_SEMICOLON_AFTER_VALUE);

        let token = self.previous.clone();
        self.chunk.add_op_print(token.line);
    }

    pub fn parse_var_declaration(&mut self) {
        self.consume(TokenType::Identifier, error::EXPECT_VARIABLE_NAME);

        let token = self.previous.clone();

        if self.match_token(TokenType::Equal) {
            self.parse_expression();
        } else {
            self.chunk.add_op_nil(token.line);
        }

        self.consume(
            TokenType::SemiColon,
            error::EXPECT_SEMICOLON_AFTER_VARIABLE_DECLARATION,
        );
        if self.scope_depth == 0 {
            let index = self.chunk.add_value(Value::String(Rc::new(token.lexeme)));
            self.chunk.add_op_define_global(index, token.line);
        } else {
            match self.resolve_local(token.lexeme.as_str()) {
                Some(_) => {
                    self.show_error(token, error::ALREADY_VARIABLE_DELCARE);
                    return;
                }
                None => {}
            };
            self.locals.push(Local {
                name: token.lexeme,
                depth: self.scope_depth,
            })
        }
    }

    pub fn resolve_local(&mut self, name: &str) -> Option<usize> {
        self.locals
            .iter()
            .rev()
            .position(|local| if local.name == name { true } else { false })
    }

    pub fn parse_variable(&mut self, precedence: Precedence) {
        let token = self.previous.clone();
        let index = self
            .resolve_local(token.lexeme.as_str())
            .map(|v| v as i32)
            .unwrap_or(-1);

        // ? Handle global
        if index == -1 {
            let global_index = self.chunk.add_value(Value::String(Rc::new(token.lexeme)));
            if precedence <= Precedence::Assignment && self.match_token(TokenType::Equal) {
                self.parse_expression();
                self.chunk.add_op_set_global(global_index, token.line);
                return;
            }
            self.chunk.add_op_get_global(global_index, token.line);
        } else {
            if precedence <= Precedence::Assignment && self.match_token(TokenType::Equal) {
                self.parse_expression();
                self.chunk.add_op_set_local(index as usize, token.line);
                return;
            }
            self.chunk.add_op_get_local(index as usize, token.line);
        }
    }

    pub fn parse_declaration(&mut self) {
        match self.current.token_type {
            TokenType::Var => {
                self.advance();
                self.parse_var_declaration();
            }
            _ => self.parse_statement(),
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    pub fn synchronize(&mut self) {
        self.panic_mode = false;

        loop {
            match self.current.token_type {
                TokenType::SemiColon => {
                    self.advance();
                    break;
                }
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => break,
                _ => {}
            }
            self.advance();
        }
    }

    pub fn parse_prefix(&mut self, precedence: Precedence) {
        let token = self.previous.clone();
        match token.token_type {
            TokenType::LeftParen => self.parse_group(),
            TokenType::Minus => self.parse_unary(),
            TokenType::Number => self.parse_number(),
            TokenType::True | TokenType::False | TokenType::Nil => self.parse_literal(),
            TokenType::String => self.parse_string(),
            TokenType::Identifier => self.parse_variable(precedence),
            _ => {
                self.show_error(token, error::EXPECT_EXPRESSION);
            }
        }
    }

    pub fn parse_and(&mut self) {
        let then_index = self.chunk.add_op_juml_if_false(0, self.previous.line);
        self.chunk.add_op_pop(self.previous.line);

        self.parse_precedence(Precedence::And);

        self.patch_op(then_index);
    }

    pub fn parse_or(&mut self) {
        let else_index = self.chunk.add_op_juml_if_false(0, self.previous.line);
        let then_index = self.chunk.add_op_jump(0, self.previous.line);
        self.patch_op(else_index);
        self.chunk.add_op_pop(self.previous.line);
        self.parse_precedence(Precedence::Or);
        self.patch_op(then_index);
    }

    pub fn parse_infix(&mut self) {
        let token = self.previous.clone();
        match token.token_type {
            TokenType::Minus
            | TokenType::Plus
            | TokenType::Star
            | TokenType::Slash
            | TokenType::EqualEqual
            | TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => self.parse_binary(),
            TokenType::And => self.parse_and(),
            TokenType::Or => self.parse_or(),
            _ => {
                panic!("Error infix parse")
            }
        }
    }

    pub fn match_token(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            false
        } else {
            self.advance();
            true
        }
    }

    pub fn check(&mut self, token_type: TokenType) -> bool {
        self.current.token_type == token_type
    }
}
