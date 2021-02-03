use std::result;

use crate::chunk::{Chunk, Value};
use crate::error;
use crate::op_code::OpCode;

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
                    let value = self.chunk.values[*index];
                    self.stack.push(value);
                }
                OpCode::OpNegate => {
                    let value = self.get_stack_value()?;
                    self.stack.push(-value);
                }
                OpCode::OpAdd => {
                    let right_value = self.get_stack_value()?;
                    let left_value = self.get_stack_value()?;
                    self.stack.push(left_value + right_value);
                }
                OpCode::OpSubtract => {
                    let right_value = self.get_stack_value()?;
                    let left_value = self.get_stack_value()?;
                    self.stack.push(left_value - right_value);
                }
                OpCode::OpMultiply => {
                    let right_value = self.get_stack_value()?;
                    let left_value = self.get_stack_value()?;
                    self.stack.push(left_value * right_value);
                }
                OpCode::OpDivide => {
                    let right_value = self.get_stack_value()?;
                    let left_value = self.get_stack_value()?;
                    self.stack.push(left_value / right_value);
                }
                _ => println!("Executing {}", code),
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
    pub fn get_stack_value(&mut self)-> Result<Value> {
        self.stack.pop().ok_or(VmError::RuntimeError(error::EMPTY_STACK))
    }
}
