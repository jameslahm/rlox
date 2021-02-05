use std::result;
use std::{collections::HashMap, rc::Rc};

use crate::error;
use crate::op_code::OpCode;
use crate::{
    binary_op,
    chunk::{Chunk, Value},
};

pub struct VM<'a> {
    pub chunk: &'a Chunk,
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
}

pub enum VmError {
    CompileError(String),
    RuntimeError(String),
}

pub type Result<T> = result::Result<T, VmError>;

impl<'a> VM<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        VM {
            chunk: chunk,
            stack: vec![],
            globals: HashMap::new(),
        }
    }
    pub fn interpret(&mut self) -> Result<()> {
        for (index, code) in self.chunk.codes.iter().enumerate() {
            self.show_stack();
            self.chunk.disassemble_op_code(code, index);
            match code {
                OpCode::OpReturn => {}
                OpCode::OpConstant(index) => {
                    let value = self.chunk.values[*index].clone();
                    self.stack.push(value);
                }
                OpCode::OpNegate => {
                    let value = self.get_stack_value()?;
                    if let Value::Double(v) = value {
                        self.stack.push(Value::Double(-v))
                    } else {
                        return Err(VmError::RuntimeError(error::OPERAND_MUST_BE_NUMBER.to_owned()));
                    }
                }
                OpCode::OpAdd => {
                    if let Value::String(left_v) = self.peek(0) {
                        if let Value::String(right_v) = self.peek(1) {
                            self.get_stack_value()?;
                            self.get_stack_value()?;

                            self.stack
                                .push(Value::String(Rc::new((*left_v).clone() + &right_v)));
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
                OpCode::OpPrint => {
                    println!("{}", self.get_stack_value()?);
                }
                OpCode::OpPop => {
                    self.get_stack_value()?;
                }
                OpCode::OpDefineGlobal(index) => {
                    let name_value = self.chunk.values[*index].clone();
                    if let Value::String(name) = name_value {
                        let value = self.get_stack_value()?;
                        self.globals.insert((*name).clone(), value);
                    } else {
                        panic!("Warn OpDefineGlobal should be string")
                    }
                }
                OpCode::OpGetGlobal(index) => {
                    let name_value = self.chunk.values[*index].clone();
                    if let Value::String(name) = name_value {
                        let message = format!("{} {}", error::UNDEFINED_VARIABLE, name);
                        // let value = self.globals.get(&(*name)).ok_or(VmError::RuntimeError(
                        //     &message))?;
                    }
                }
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
            .ok_or(VmError::RuntimeError(error::EMPTY_STACK.to_owned()))
    }

    pub fn peek(&self, distance: usize) -> Value {
        let stack_len = self.stack.len();
        if stack_len < distance + 1 {
            panic!("Error peek stack")
        }
        self.stack[self.stack.len() - 1 - distance].clone()
    }
}
