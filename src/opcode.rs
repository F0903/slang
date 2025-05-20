#[repr(u8)]
#[derive(Debug, Clone)]
pub enum OpCode {
    SetLocal,
    GetLocal,
    Pop,
    PopN,
    SetGlobal,
    GetGlobal,
    DefineGlobal,
    Constant,
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

impl OpCode {
    pub const fn to_code(self) -> u8 {
        unsafe { std::mem::transmute(self) }
    }

    pub const fn from_code(code: u8) -> Self {
        unsafe { std::mem::transmute(code) }
    }
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        Self::from_code(value)
    }
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        value.to_code()
    }
}
