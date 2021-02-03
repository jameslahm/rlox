use std::vec;

use crate::op_code::OpCode;

pub type Value = f64;

pub struct Chunk {
    pub codes: Vec<OpCode>,
    pub values: Vec<Value>,
    pub lines: Vec<i32>,
}

impl Chunk {
    pub fn new()->Self {
        Chunk {
            codes:vec![],
            values:vec![],
            lines:vec![]
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
    pub fn add_op_return(&mut self,line:i32) {
        self.codes.push(OpCode::OpReturn);
        self.lines.push(line);
    }
    pub fn add_op_constant(&mut self, value: Value,line:i32) {
        self.values.push(value);
        let index = self.values.len() - 1;
        self.codes.push(OpCode::OpConstant(index));
        self.lines.push(line);
    }
    pub fn add_op_negate(&mut self,line:i32){
        self.codes.push(OpCode::OpNegate);
        self.lines.push(line);
    }
}
