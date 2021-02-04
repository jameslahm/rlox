use std::rc::Rc;
use std::result;

use crate::error;
use crate::op_code::OpCode;
use crate::{
    binary_op,
    chunk::{Chunk, Value},
};

pub struct VM<'a> {
    pub chunk: &'a Chunk,
    pub stack: Vec<Value>,
}

pub enum VmError {
    CompileError(&'static str),
    RuntimeError(&'static str),
}

pub type Result<T> = result::Result<T, VmError>;

impl<'a> VM<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        VM {
            chunk: chunk,
            stack: vec![],
        }
    }
    pub fn interpret(&mut self) -> Result<()> {
        for (index, code) in self.chunk.codes.iter().enumerate() {
            self.show_stack();
            self.chunk.disassemble_op_code(code, index);
            match code {
                OpCode::OpReturn => {
                    let value = self.get_stack_value()?;
                    println!("{}", value);
                    return Ok(());
                }
                OpCode::OpConstant(index) => {
                    let value = self.chunk.values[*index].clone();
                    self.stack.push(value);
                }
                OpCode::OpNegate => {
                    let value = self.get_stack_value()?;
                    if let Value::Double(v) = value {
                        self.stack.push(Value::Double(-v))
                    } else {
                        return Err(VmError::RuntimeError(error::OPERAND_MUST_BE_NUMBER));
                    }
                }
                OpCode::OpAdd => {
                    if let Value::String(left_v) = self.peek(0) {
                        if let Value::String(right_v) = self.peek(1) {
                            self.get_stack_value()?;
                            self.get_stack_value()?;
                            
                            
                            self.stack.push(Value::String(Rc::new((*left_v).clone() + &right_v)));
                            continue;
                        }
                    }
                    binary_op!(self,Double,+);
                }
                OpCode::OpSubtract => {
                    binary_op!(self,Double,-);
                }
                OpCode::OpMultiply => {
                    binary_op!(self,Double,*);
                }
                OpCode::OpDivide => {
                    binary_op!(self,Double,/);
                }
                OpCode::OpNil => {
                    self.stack.push(Value::Nil);
                }
                OpCode::OpTrue => {
                    self.stack.push(Value::Bool(true));
                }
                OpCode::OpFalse => {
                    self.stack.push(Value::Bool(false));
                }
                OpCode::OpNot => {
                    let boolean: bool = self.get_stack_value()?.into();
                    self.stack.push(Value::Bool(boolean));
                }
                OpCode::OpEqual => {
                    let left_value = self.get_stack_value()?;
                    let right_value = self.get_stack_value()?;
                    self.stack.push(Value::Bool(left_value == right_value));
                }
                OpCode::OpGreater => {
                    binary_op!(self,Bool,>);
                }
                OpCode::OpLess => {
                    binary_op!(self,Bool,<);
                } // _ => println!("Executing {}", code),
            }
        }

        Ok(())
    }
    pub fn show_stack(&self) {
        print!("        ");
        for value in self.stack.iter() {
            print!("[ {} ]", value)
        }
        println!()
    }
    pub fn get_stack_value(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .ok_or(VmError::RuntimeError(error::EMPTY_STACK))
    }

    pub fn peek(&self, distance: usize) -> Value {
        let stack_len = self.stack.len();
        if stack_len < distance + 1 {
            panic!("Error peek stack")
        }
        self.stack[self.stack.len() - 1 - distance].clone()
    }
}
