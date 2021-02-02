use std::result;

use crate::chunk::Chunk;
use crate::op_code::OpCode;

pub struct VM<'a> {
    pub chunk: &'a Chunk,
}

pub enum VmError {
    CompileError,
    RuntimeError,
}

pub type Result = result::Result<(), VmError>;

impl<'a> VM<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        VM { chunk: chunk }
    }
    pub fn interpret(&self) -> Result {
        for code in self.chunk.codes.iter() {
            match code {
                OpCode::OpReturn => {
                    return Ok(());
                },
                OpCode::OpConstant(index) => {
                    let value = self.chunk.values[*index];
                    println!("{}",value);
                },
                _ => println!("Executing {}",code)
            }
        }

        Ok(())
    }
}
