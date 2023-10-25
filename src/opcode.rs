#[repr(u8)]
#[derive(Debug)]
pub enum OpCode {
    Constant,
    ConstantLong,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
