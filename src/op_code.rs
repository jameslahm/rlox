use std::fmt;

#[derive(Debug)]
pub enum OpCode {
    OpReturn,
    OpConstant(usize),
    OpNegate,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
    OpNil,
    OpTrue,
    OpFalse,
    OpNot,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpCode::OpReturn => write!(f, "OpReturn"),
            OpCode::OpConstant(i) => write!(f, "OpConstant {}", i),
            OpCode::OpNegate => write!(f,"OpNegate"),
            OpCode::OpAdd =>write!(f,"OpAdd"),
            OpCode::OpSubtract => write!(f,"OpSubtract"),
            OpCode::OpMultiply => write!(f,"OpMultiply"),
            OpCode::OpDivide => write!(f,"OpDivide"),
            OpCode::OpNil => write!(f,"OpNil"),
            OpCode::OpTrue =>write!(f,"OpTrue"),
            OpCode::OpFalse =>write!(f,"OpFalse"),
            OpCode::OpNot =>write!(f,"OpNot")
            // _ => write!(f, "Unknown OpCode...\n"),
        }
    }
}

pub fn test() {
    let mut chunk = vec![OpCode::OpReturn];
    chunk.push(OpCode::OpConstant(1));
    println!("{:?}", chunk)
}
