use core::panic;
use num::FromPrimitive;
use num_derive::FromPrimitive;
use std::{ops::Add, rc::Rc, vec};

use crate::{chunk::{Chunk, Closure, Function, Value}, error, scanner::Scanner, token::{Token, TokenType}};

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
            TokenType::LeftParen => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    TokenError,
    ConsumeError(String),
}

#[derive(Debug,Clone)]
pub struct Local {
    pub name: String,
    pub depth: u32,
}

#[derive(Debug,Clone)]
pub struct Builder {
    pub chunk: Chunk,
    pub scope_depth: u32,
    pub locals: Vec<Local>,
}

impl Builder {
    fn new(name: String) -> Builder {
        let mut builder = Builder {
            chunk:Chunk::new(),
            scope_depth:0,
            locals:vec![]
        };
        builder.locals.push(Local {
            name: name,
            depth: 0,
        });
        builder
    }
}

pub struct Compiler {
    pub previous: Token,
    pub current: Token,
    pub scanner: Scanner,
    pub panic_mode: bool,
    pub errors: Vec<ParseError>,
    pub builder: Builder,
}

impl Compiler {
    pub fn new(source: String) -> Self {
        Compiler {
            previous: Token::default(),
            current: Token::default(),
            panic_mode: false,
            scanner: Scanner::new(source),
            errors: vec![],
            builder: Builder::new("".to_owned()),
        }
    }

    pub fn compile(&mut self) -> Function {
        self.advance();
        while !self.match_token(TokenType::Eof) {
            self.parse_declaration();
        }
        self.consume(TokenType::Eof, error::EXPECT_EOF);
        Function::new(0,self.builder.chunk.clone(),"".to_owned())
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
        self.builder
            .chunk
            .add_op_constant(value, self.previous.line);
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
                self.builder.chunk.add_op_negate(token.line);
            }
            TokenType::Bang => {
                self.builder.chunk.add_op_not(token.line);
            }
            _ => {}
        }
    }

    pub fn parse_binary(&mut self) {
        let token: Token = self.previous.clone();

        let precedence: Precedence = token.token_type.into();
        self.parse_precedence(precedence);
        match token.token_type {
            TokenType::Plus => self.builder.chunk.add_op_add(token.line),
            TokenType::Minus => self.builder.chunk.add_op_subtract(token.line),
            TokenType::Star => self.builder.chunk.add_op_multily(token.line),
            TokenType::Slash => self.builder.chunk.add_op_divide(token.line),
            TokenType::BangEqual => {
                self.builder.chunk.add_op_equal(token.line);
                self.builder.chunk.add_op_not(token.line);
            }
            TokenType::EqualEqual => {
                self.builder.chunk.add_op_equal(token.line);
            }
            TokenType::Greater => {
                self.builder.chunk.add_op_greater(token.line);
            }
            TokenType::GreaterEqual => {
                self.builder.chunk.add_op_less(token.line);
                self.builder.chunk.add_op_not(token.line);
            }
            TokenType::Less => {
                self.builder.chunk.add_op_less(token.line);
            }
            TokenType::LessEqual => {
                self.builder.chunk.add_op_greater(token.line);
                self.builder.chunk.add_op_not(token.line);
            }
            _ => {}
        }
    }

    pub fn parse_literal(&mut self) {
        let token = self.previous.clone();
        match token.token_type {
            TokenType::False => self.builder.chunk.add_op_false(token.line),
            TokenType::True => self.builder.chunk.add_op_true(token.line),
            TokenType::Nil => self.builder.chunk.add_op_nil(token.line),
            _ => {}
        }
    }

    pub fn parse_expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    pub fn parse_string(&mut self) {
        let token = self.previous.clone();
        self.builder
            .chunk
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
            TokenType::Return => {
                self.parse_return_statement();
            }
            _ => self.parse_expression_statement(),
        }
    }

    pub fn parse_return_statement(&mut self){
        if self.match_token(TokenType::SemiColon){
            self.builder.chunk.add_op_nil(self.previous.line);
            self.builder.chunk.add_op_return(self.previous.line);
        } else {
            self.parse_expression();
            self.consume(TokenType::SemiColon, error::EXPECT_SEMICOLON_AFTER_RETURN);
            self.builder.chunk.add_op_return(self.previous.line);
        }
    }

    pub fn parse_for_statement(&mut self) {
        self.enter_scope();
        self.consume(TokenType::LeftParen, error::EXPECT_LEFT_PAREN_AFTER_FOR);
        if self.match_token(TokenType::SemiColon) {
        } else if self.match_token(TokenType::Var) {
            self.parse_var_declaration();
        } else {
            self.parse_expression_statement();
        }

        let mut exit_index: i32 = -1;
        let condition_index = self.builder.chunk.codes.len();
        if !self.match_token(TokenType::SemiColon) {
            self.parse_expression();
            self.consume(TokenType::SemiColon, error::EXPECT_SEMICOLON_AFTER_LOOP);

            exit_index = self
                .builder
                .chunk
                .add_op_juml_if_false(0, self.previous.line) as i32;
            self.builder.chunk.add_op_pop(self.previous.line);
        }

        let incre_index = self.builder.chunk.codes.len();

        if !self.match_token(TokenType::RightParen) {
            let body_index = self.builder.chunk.add_op_jump(0, self.previous.line);
            self.parse_expression();
            self.builder.chunk.add_op_pop(self.previous.line);
            self.builder
                .chunk
                .add_op_loop(condition_index, self.previous.line);
            self.patch_op(body_index);
        }

        self.builder
            .chunk
            .add_op_loop(incre_index, self.previous.line);

        if exit_index != -1 {
            self.patch_op(exit_index as usize);
            self.builder.chunk.add_op_pop(self.previous.line);
        }
        self.exit_scope();
    }

    pub fn parse_while_statement(&mut self) {
        let loop_index = self.builder.chunk.codes.len();

        self.consume(TokenType::LeftParen, error::EXPECT_LEFT_PAREN_AFTER_WHILE);
        self.parse_expression();
        self.consume(
            TokenType::RightParen,
            error::EXPECT_RIGHT_PAREN_AFTER_CONDITION,
        );

        let exit_index = self
            .builder
            .chunk
            .add_op_juml_if_false(0, self.previous.line);
        self.builder.chunk.add_op_pop(self.previous.line);
        self.parse_statement();
        self.builder.chunk.add_op_loop(
            self.builder.chunk.codes.len() - loop_index,
            self.previous.line,
        );

        self.patch_op(exit_index);
        self.builder.chunk.add_op_pop(self.previous.line);
    }

    pub fn parse_if_statement(&mut self) {
        self.consume(TokenType::LeftParen, error::EXPECT_LEFT_PAREN_AFTER_IF);
        self.parse_expression();
        self.consume(
            TokenType::RightParen,
            error::EXPECT_RIGHT_PAREN_AFTER_CONDITION,
        );

        let then_index = self
            .builder
            .chunk
            .add_op_juml_if_false(0, self.previous.line);
        self.builder.chunk.add_op_pop(self.previous.line);
        self.parse_statement();

        let else_index = self.builder.chunk.add_op_jump(0, self.previous.line);

        self.patch_op(then_index);
        self.builder.chunk.add_op_pop(self.previous.line);

        if self.match_token(TokenType::Else) {
            self.parse_statement();
        }
        self.patch_op(else_index);
    }

    pub fn patch_op(&mut self, index: usize) {
        let code_len = self.builder.chunk.codes.len();
        let op = &mut self.builder.chunk.codes[index];
        match op {
            OpCode::OpJumpIfFalse(ref mut offset) => {
                *offset = code_len - index;
            }
            OpCode::OpJump(ref mut offset) => {
                *offset = code_len - index;
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
        self.builder.scope_depth += 1;
    }
    pub fn exit_scope(&mut self) {
        self.builder.scope_depth -= 1;
        while self.builder.locals[self.builder.locals.len() - 1].depth > self.builder.scope_depth {
            self.builder.locals.remove(self.builder.locals.len());
            self.builder.chunk.add_op_pop(self.previous.line);
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
        self.builder.chunk.add_op_print(token.line);
    }

    pub fn parse_var_declaration(&mut self) {
        self.consume(TokenType::Identifier, error::EXPECT_VARIABLE_NAME);

        let token = self.previous.clone();

        if self.match_token(TokenType::Equal) {
            self.parse_expression();
        } else {
            self.builder.chunk.add_op_nil(token.line);
        }

        self.consume(
            TokenType::SemiColon,
            error::EXPECT_SEMICOLON_AFTER_VARIABLE_DECLARATION,
        );

        self.define_variable(token);
    }

    pub fn define_local_variable(&mut self, token: Token) {
        match self.resolve_local(token.lexeme.as_str()) {
            Some(_) => {
                self.show_error(token, error::ALREADY_VARIABLE_DELCARE);
                return;
            }
            None => {}
        };
        self.builder.locals.push(Local {
            name: token.lexeme,
            depth: self.builder.scope_depth,
        })
    }

    pub fn define_global_variable(&mut self, token: Token) {
        let index = self
            .builder
            .chunk
            .add_value(Value::String(Rc::new(token.lexeme)));
        self.builder.chunk.add_op_define_global(index, token.line);
    }

    pub fn define_variable(&mut self, token: Token) {
        if self.builder.scope_depth == 0 {
            self.define_global_variable(token);
        } else {
            self.define_local_variable(token);
        }
    }

    pub fn resolve_local(&mut self, name: &str) -> Option<usize> {
        self.builder
            .locals
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
            let global_index = self
                .builder
                .chunk
                .add_value(Value::String(Rc::new(token.lexeme)));
            if precedence <= Precedence::Assignment && self.match_token(TokenType::Equal) {
                self.parse_expression();
                self.builder
                    .chunk
                    .add_op_set_global(global_index, token.line);
                return;
            }
            self.builder
                .chunk
                .add_op_get_global(global_index, token.line);
        } else {
            if precedence <= Precedence::Assignment && self.match_token(TokenType::Equal) {
                self.parse_expression();
                self.builder
                    .chunk
                    .add_op_set_local(index as usize, token.line);
                return;
            }
            self.builder
                .chunk
                .add_op_get_local(index as usize, token.line);
        }
    }

    pub fn parse_func_declaration(&mut self) {
        self.consume(TokenType::Identifier, error::EXPECT_FUNCTION_NAME);
        let token = self.previous.clone();
        if self.builder.scope_depth != 0 {
            self.define_variable(token.clone());
        }

        let origin_builder = self.builder.clone();
        self.builder = Builder::new(token.lexeme.clone());

        self.enter_scope();

        self.consume(
            TokenType::LeftParen,
            error::EXPECT_LEFT_PAREN_AFTER_FUNCTION,
        );
        let mut arity = 0;
        if !self.check(TokenType::RightParen) {
            loop {
                arity += 1;
                self.consume(TokenType::Identifier, error::EXPECT_PARAMETER_NAME);
                self.define_local_variable(self.previous.clone());
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(
            TokenType::RightParen,
            error::EXPECT_RIGHT_PAREN_AFTER_PARAMETERS,
        );

        self.consume(
            TokenType::LeftBrace,
            error::EXPECT_LEFT_BRACE_BEFORE_FUNCTION_BODY,
        );
        self.parse_block_statement();

        self.builder.chunk.add_op_nil(self.previous.line);
        self.builder.chunk.add_op_return(self.previous.line);

        self.exit_scope();

        let function: Function = Function::new(arity, self.builder.chunk.clone(), token.lexeme.clone());

        self.builder = origin_builder;
        self.builder
            .chunk
            .add_op_constant(Value::Closure(Rc::new(Closure::new(Rc::new(function)))), self.previous.line);
        if self.builder.scope_depth == 0 {
            self.define_global_variable(token.clone());
        }
    }

    pub fn parse_declaration(&mut self) {
        match self.current.token_type {
            TokenType::Var => {
                self.advance();
                self.parse_var_declaration();
            }
            TokenType::Fun => {
                self.advance();
                self.parse_func_declaration()
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
        let then_index = self
            .builder
            .chunk
            .add_op_juml_if_false(0, self.previous.line);
        self.builder.chunk.add_op_pop(self.previous.line);

        self.parse_precedence(Precedence::And);

        self.patch_op(then_index);
    }

    pub fn parse_or(&mut self) {
        let else_index = self
            .builder
            .chunk
            .add_op_juml_if_false(0, self.previous.line);
        let then_index = self.builder.chunk.add_op_jump(0, self.previous.line);
        self.patch_op(else_index);
        self.builder.chunk.add_op_pop(self.previous.line);
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
            TokenType::LeftParen => self.parse_call(),
            _ => {
                panic!("Error infix parse")
            }
        }
    }

    pub fn parse_call(&mut self) {
        let mut arg_count = 0;
        if !self.check(TokenType::RightParen) {
            loop {
                self.parse_expression();
                arg_count += 1;
                if !self.match_token(TokenType::Comma){
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen,error::EXPECT_RIGHT_PAREN_AFTER_ARG);

        self.builder.chunk.add_op_call(arg_count, self.previous.line);
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
