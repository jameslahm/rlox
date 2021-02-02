use std::fmt;

#[derive(Debug)]
pub enum OpCode {
    OpReturn,
    OpConstant(usize),
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::OpReturn => write!(f, "OpReturn"),
            OpCode::OpConstant(i) => write!(f, "OpConstant {}", i),
            _ => write!(f,"Unknown OpCode...\n")
        }
    }
}

pub fn test() {
    let mut chunk = vec![OpCode::OpReturn];
    chunk.push(OpCode::OpConstant(1));
    println!("{:?}", chunk)
}
