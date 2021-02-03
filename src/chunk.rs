use std::fmt::{Formatter, Result};
use std::{fmt::Display, vec};

use crate::op_code::OpCode;

#[derive(Debug,Clone, Copy)]
pub enum Value {
    Bool(bool),
    Double(f64),
    Nil,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Bool(v) => {
                let other_boolean:bool = other.to_owned().into();
                *v == other_boolean
            }
            Value::Double(f) => {
                let other_f:f64 = other.to_owned().into();
                *f == other_f
            }
            Value::Nil => {
                if let Value::Nil = other {
                    true
                } else {
                    false
                }
            }
        }
    }
}


impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(v) => v,
            Value::Nil => false,
            _ => true
        }
    }
}



impl From<Value> for f64 {
    fn from(value: Value) -> Self {
        match value {
            Value::Double(f)=>f,
            Value::Bool(v) => (v as i32) as f64,
            Value::Nil => 0.0
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Value::Bool(v) => write!(f, "Bool {}", v),
            Value::Double(v) => write!(f, "Double {}", v),
            Value::Nil => write!(f, "Nil"),
        }
    }
}

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

    pub fn add_op_false(&mut self,line: i32){
        self.codes.push(OpCode::OpFalse);
        self.lines.push(line);
    }

    pub fn add_op_true(&mut self,line: i32) {
        self.codes.push(OpCode::OpTrue);
        self.lines.push(line);
    }

    pub fn add_op_nil(&mut self,line:i32){
        self.codes.push(OpCode::OpNil);
        self.lines.push(line);
    }

    pub fn add_op_not(&mut self,line:i32){
        self.codes.push(OpCode::OpNot);
        self.lines.push(line);
    }

    pub fn add_op_equal(&mut self,line:i32){
        self.codes.push(OpCode::OpEqual);
        self.lines.push(line);
    }

    pub fn add_op_greater(&mut self,line:i32){
        self.codes.push(OpCode::OpGreater);
        self.lines.push(line);
    }

    pub fn add_op_less(&mut self,line:i32){
        self.codes.push(OpCode::OpLess);
        self.lines.push(line);
    }

}
