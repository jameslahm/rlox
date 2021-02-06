use std::{cell::RefCell, result};
use std::{collections::HashMap, rc::Rc};

use crate::op_code::OpCode;
use crate::{binary_op, chunk::Value};
use crate::{chunk::Function, error};

pub struct VM {
    pub stack: Rc<RefCell<Vec<Value>>>,
    pub globals: HashMap<String, Value>,
    pub frames: Vec<CallFrame>,
}

pub struct CallFrame {
    pub function: Rc<Function>,
    pub ip: usize,
    pub slots: Rc<RefCell<Vec<Value>>>,
    pub base: usize,
}

impl<'a> CallFrame {
    fn new(function: Rc<Function>, stack: Rc<RefCell<Vec<Value>>>, base: usize) -> CallFrame {
        CallFrame {
            function: function,
            ip: 0,
            slots: stack,
            base: base,
        }
    }
    pub fn show_stack(&self) {
        print!("        ");
        for value in self.slots.borrow().iter() {
            print!("[ {} ]", value)
        }
        println!()
    }

    pub fn get_stack_value(&mut self) -> Result<Value> {
        self.slots
            .borrow_mut()
            .pop()
            .ok_or(VmError::RuntimeError(error::EMPTY_STACK.to_owned()))
    }

    pub fn peek(&self, distance: usize) -> Value {
        let slots_len = self.slots.borrow().len();
        if slots_len < distance + 1 {
            panic!("Error peek slots")
        }
        self.slots.borrow()[slots_len - 1 - distance].clone()
    }
}

pub enum VmError {
    CompileError(String),
    RuntimeError(String),
}

pub type Result<T> = result::Result<T, VmError>;

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Rc::new(RefCell::new(vec![])),
            globals: HashMap::new(),
            frames: vec![],
        }
    }
    pub fn interpret(&mut self, function: Rc<Function>) -> Result<()> {
        let global_frame = CallFrame::new(function, self.stack.clone(), 0);
        self.frames.push(global_frame);

        let mut frame = &mut self.frames[0];
        while frame.ip < frame.function.chunk.codes.len() {
            let code = frame.function.chunk.codes[frame.ip];
            frame.show_stack();
            frame.function.chunk.disassemble_op_code(&code, frame.ip);
            match code {
                OpCode::OpConstant(index) => {
                    let value = frame.function.chunk.values[index].clone();
                    frame.slots.borrow_mut().push(value);
                }
                OpCode::OpNegate => {
                    let value = frame.get_stack_value()?;
                    if let Value::Double(v) = value {
                        frame.slots.borrow_mut().push(Value::Double(-v))
                    } else {
                        return Err(VmError::RuntimeError(
                            error::OPERAND_MUST_BE_NUMBER.to_owned(),
                        ));
                    }
                }
                OpCode::OpAdd => {
                    if let Value::String(left_v) = frame.peek(0) {
                        if let Value::String(right_v) = frame.peek(1) {
                            frame.get_stack_value()?;
                            frame.get_stack_value()?;

                            frame
                                .slots
                                .borrow_mut()
                                .push(Value::String(Rc::new((*left_v).clone() + &right_v)));
                        }
                    } else {
                        binary_op!(frame,Double,+);
                    }
                }
                OpCode::OpSubtract => {
                    binary_op!(frame,Double,-);
                }
                OpCode::OpMultiply => {
                    binary_op!(frame,Double,*);
                }
                OpCode::OpDivide => {
                    binary_op!(frame,Double,/);
                }
                OpCode::OpNil => {
                    frame.slots.borrow_mut().push(Value::Nil);
                }
                OpCode::OpTrue => {
                    frame.slots.borrow_mut().push(Value::Bool(true));
                }
                OpCode::OpFalse => {
                    frame.slots.borrow_mut().push(Value::Bool(false));
                }
                OpCode::OpNot => {
                    let boolean: bool = frame.get_stack_value()?.into();
                    frame.slots.borrow_mut().push(Value::Bool(boolean));
                }
                OpCode::OpEqual => {
                    let left_value = frame.get_stack_value()?;
                    let right_value = frame.get_stack_value()?;
                    frame
                        .slots
                        .borrow_mut()
                        .push(Value::Bool(left_value == right_value));
                }
                OpCode::OpGreater => {
                    binary_op!(frame,Bool,>);
                }
                OpCode::OpLess => {
                    binary_op!(frame,Bool,<);
                } // _ => println!("Executing {}", code),
                OpCode::OpPrint => {
                    println!("{}", frame.get_stack_value()?);
                }
                OpCode::OpPop => {
                    frame.get_stack_value()?;
                }
                OpCode::OpDefineGlobal(index) => {
                    let name_value = frame.function.chunk.values[index].clone();
                    if let Value::String(name) = name_value {
                        let value = frame.get_stack_value()?;
                        self.globals.insert((*name).clone(), value);
                    } else {
                        panic!(error::WARN_GLOBAL_BE_STRING);
                    }
                }
                OpCode::OpGetGlobal(index) => {
                    let name_value = frame.function.chunk.values[index].clone();
                    if let Value::String(name) = name_value {
                        let message = format!("{} {}", error::UNDEFINED_VARIABLE, name);
                        let value = self
                            .globals
                            .get(&(*name))
                            .ok_or(VmError::RuntimeError(message))?;
                        frame.slots.borrow_mut().push(value.clone());
                    } else {
                        panic!(error::WARN_GLOBAL_BE_STRING);
                    }
                }
                OpCode::OpSetGlobal(index) => {
                    let name_value = frame.function.chunk.values[index].clone();
                    if let Value::String(name) = name_value {
                        let message = format!("{} {}", error::UNDEFINED_VARIABLE, name);
                        let assign_value = frame.get_stack_value()?;
                        let value = self
                            .globals
                            .get_mut(&(*name))
                            .ok_or(VmError::RuntimeError(message))?;
                        *value = assign_value;
                        frame.slots.borrow_mut().push(value.clone());
                    } else {
                        panic!(error::WARN_GLOBAL_BE_STRING);
                    }
                }
                OpCode::OpGetLocal(index) => {
                    frame
                        .slots
                        .borrow_mut()
                        .push(frame.slots.borrow()[frame.base + index].clone());
                }
                OpCode::OpSetLocal(index) => {
                    frame.slots.borrow_mut()[frame.base + index] = frame.peek(0);
                }
                OpCode::OpJumpIfFalse(index) => {
                    let boolean: bool = frame.peek(0).into();
                    if !boolean {
                        frame.ip += index;
                        continue;
                    }
                }
                OpCode::OpJump(index) => {
                    frame.ip += index;
                    continue;
                }
                OpCode::OpLoop(index) => {
                    frame.ip -= index;
                    continue;
                }
                OpCode::OpCall(arg_count) => {
                    let value = frame.peek(arg_count);
                    match value {
                        Value::Function(function) => {
                            if function.arity != arg_count {
                                return Err(VmError::RuntimeError(format!(
                                    "Expected {} arguments but got {}",
                                    function.arity, arg_count
                                )))
                            }
                            let new_frame = CallFrame::new(
                                function.clone(),
                                self.stack.clone(),
                                self.stack.borrow().len() - arg_count - 1,
                            );
                            self.frames.push(new_frame);
                            let frame_len = self.frames.len();
                            frame = &mut self.frames[frame_len - 1];
                            continue;
                        }
                        Value::NativeFunction(function)=>{
                            let value = function();
                            frame.get_stack_value()?;
                            frame.slots.borrow_mut().push(value);
                        }
                        _ => {
                            return Err(VmError::RuntimeError("Not a callable".to_owned()));
                        }
                    }
                }
                OpCode::OpReturn => {
                    let value = frame.get_stack_value()?;
                    let base = frame.base;
                    self.stack.borrow_mut().drain(base..);

                    self.stack.borrow_mut().push(value);

                    self.frames.pop();
                    let frame_len = self.frames.len();
                    if frame_len==0 {
                        return  Ok(());
                    } else {
                        frame = & mut self.frames[frame_len-1];
                    }
                }
            }
            frame.ip += 1;
        }

        Ok(())
    }
}
