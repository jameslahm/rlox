use crate::op_code::OpCode;

type Value = f64;

pub struct Chunk {
    pub codes: Vec<OpCode>,
    pub values: Vec<Value>,
    pub lines: Vec<i32>,
}

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==\n", name);
        for (index, code) in self.codes.iter().enumerate() {
            self.dis
        }
    }
    pub fn disassemble_op_code(&self,code:&OpCode,index:usize){
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
    pub fn add_op_return(&mut self) {
        self.codes.push(OpCode::OpReturn);
    }
    pub fn add_op_constant(&mut self,value:Value){
        self.values.push(value);
        let index = self.values.len()-1;
        self.codes.push(OpCode::OpConstant(index));
    }
}
