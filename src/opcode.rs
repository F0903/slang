#[repr(u8)]
#[derive(Debug, Clone)]
pub enum OpCode {
    Constant,
    ConstantLong,
    None,
    True,
    False,
    Is,
    IsNot,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Not,
    Return,
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
