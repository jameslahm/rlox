use std::fmt::Debug;
use std::{fmt::Display, vec};
use std::{
    fmt::{Formatter, Result},
    rc::Rc,
};

use crate::op_code::OpCode;

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: String,
}

impl Function {
    pub fn new(arity: usize, chunk: Chunk, name: String) -> Function {
        Function {
            arity: arity,
            chunk: chunk,
            name: name,
        }
    }
}

pub struct CallFrame<'a> {
    pub functinon: Rc<Function>,
    pub ip: i32,
    pub slots: &'a Vec<Value>,

    // stack base
    pub base: i32,
}

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Double(f64),
    Nil,
    String(Rc<String>),
    Function(Rc<Function>),
    NativeFunction(Box<fn()->Value>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(left_v), Value::Bool(right_v)) => left_v == right_v,
            (Value::Double(left_v), Value::Double(right_v)) => left_v == right_v,
            (Value::Nil, Value::Nil) => true,
            (Value::String(left_v), Value::String(right_v)) => left_v == right_v,
            _ => false,
        }
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(v) => v,
            Value::Nil => false,
            _ => true,
        }
    }
}

impl From<Value> for f64 {
    fn from(value: Value) -> Self {
        match value {
            Value::Double(f) => f,
            Value::Bool(v) => (v as i32) as f64,
            Value::Nil => 0.0,
            _ => 0.0,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Value::Bool(v) => write!(f, "Bool {}", v),
            Value::Double(v) => write!(f, "Double {}", v),
            Value::Nil => write!(f, "Nil"),
            Value::String(b) => write!(f, "{}", b),
            Value::Function(function) => write!(f, "{:?}", function),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub codes: Vec<OpCode>,
    pub values: Vec<Value>,
    pub lines: Vec<i32>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            codes: vec![],
            values: vec![],
            lines: vec![],
        }
    }
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==\n", name);
        for (index, code) in self.codes.iter().enumerate() {
            self.disassemble_op_code(code, index);
        }
    }
    pub fn disassemble_op_code(&self, code: &OpCode, index: usize) {
        print!("{:04}  ", index);

        if index > 0 && self.lines[index] == self.lines[index - 1] {
            print!("    | ")
        } else {
            print!("{:04}", self.lines[index])
        }
        match code {
            OpCode::OpConstant(i) => println!("{} {} '{}'", code, i, self.values[*i]),
            _ => println!("{}", code),
        }
    }
    pub fn add_op_return(&mut self, line: i32) {
        self.codes.push(OpCode::OpReturn);
        self.lines.push(line);
    }
    pub fn add_op_constant(&mut self, value: Value, line: i32) {
        self.values.push(value);
        let index = self.values.len() - 1;
        self.codes.push(OpCode::OpConstant(index));
        self.lines.push(line);
    }
    pub fn add_op_negate(&mut self, line: i32) {
        self.codes.push(OpCode::OpNegate);
        self.lines.push(line);
    }

    pub fn add_op_add(&mut self, line: i32) {
        self.codes.push(OpCode::OpAdd);
        self.lines.push(line);
    }

    pub fn add_op_subtract(&mut self, line: i32) {
        self.codes.push(OpCode::OpSubtract);
        self.lines.push(line);
    }

    pub fn add_op_multily(&mut self, line: i32) {
        self.codes.push(OpCode::OpMultiply);
        self.lines.push(line);
    }

    pub fn add_op_divide(&mut self, line: i32) {
        self.codes.push(OpCode::OpDivide);
        self.lines.push(line);
    }

    pub fn add_op_false(&mut self, line: i32) {
        self.codes.push(OpCode::OpFalse);
        self.lines.push(line);
    }

    pub fn add_op_true(&mut self, line: i32) {
        self.codes.push(OpCode::OpTrue);
        self.lines.push(line);
    }

    pub fn add_op_nil(&mut self, line: i32) {
        self.codes.push(OpCode::OpNil);
        self.lines.push(line);
    }

    pub fn add_op_not(&mut self, line: i32) {
        self.codes.push(OpCode::OpNot);
        self.lines.push(line);
    }

    pub fn add_op_equal(&mut self, line: i32) {
        self.codes.push(OpCode::OpEqual);
        self.lines.push(line);
    }

    pub fn add_op_greater(&mut self, line: i32) {
        self.codes.push(OpCode::OpGreater);
        self.lines.push(line);
    }

    pub fn add_op_less(&mut self, line: i32) {
        self.codes.push(OpCode::OpLess);
        self.lines.push(line);
    }

    pub fn add_op_print(&mut self, line: i32) {
        self.codes.push(OpCode::OpPrint);
        self.lines.push(line);
    }

    pub fn add_op_define_global(&mut self, index: usize, line: i32) {
        self.codes.push(OpCode::OpDefineGlobal(index));
        self.lines.push(line);
    }

    pub fn add_value(&mut self, value: Value) -> usize {
        self.values.push(value);
        self.values.len() - 1
    }

    pub fn add_op_get_global(&mut self, index: usize, line: i32) {
        self.codes.push(OpCode::OpGetGlobal(index));
        self.lines.push(line);
    }

    pub fn add_op_set_global(&mut self, index: usize, line: i32) {
        self.codes.push(OpCode::OpSetGlobal(index));
        self.lines.push(line);
    }

    pub fn add_op_pop(&mut self, line: i32) {
        self.codes.push(OpCode::OpPop);
        self.lines.push(line);
    }

    pub fn add_op_get_local(&mut self, index: usize, line: i32) {
        self.codes.push(OpCode::OpGetLocal(index));
        self.lines.push(line);
    }

    pub fn add_op_set_local(&mut self, index: usize, line: i32) {
        self.codes.push(OpCode::OpSetLocal(index));
        self.lines.push(line);
    }

    pub fn add_op_juml_if_false(&mut self, index: usize, line: i32) -> usize {
        self.codes.push(OpCode::OpJumpIfFalse(index));
        self.lines.push(line);
        return self.codes.len() - 1;
    }

    pub fn add_op_jump(&mut self, index: usize, line: i32) -> usize {
        self.codes.push(OpCode::OpJump(index));
        self.lines.push(line);
        return self.codes.len() - 1;
    }

    pub fn add_op_loop(&mut self, index: usize, line: i32) -> usize {
        self.codes.push(OpCode::OpLoop(index));
        self.lines.push(line);
        return self.codes.len() - 1;
    }
    pub fn add_op_call(&mut self, arg_count: usize, line: i32) {
        self.codes.push(OpCode::OpCall(arg_count));
        self.lines.push(line);
    }
}
