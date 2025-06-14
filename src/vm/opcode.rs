/// Each OpCode is 1 byte, but some OpCodes have arguments of varying length following them.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum OpCode {
    // 1 byte for is_local, 2 bytes for index
    Closure,
    Call,
    Backjump,
    Jump,
    JumpIfTrue,
    JumpIfFalse,
    CloseUpvalue,
    SetUpvalue,
    GetUpvalue,
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
    #[inline]
    pub const fn to_code(self) -> u8 {
        // SAFETY: OpCode is repr(u8), so we can safely transmute it to u8.
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    pub const fn from_code(code: u8) -> Self {
        // SAFETY: OpCode is repr(u8), so we can safely transmute u8 to OpCode.
        unsafe { std::mem::transmute(code) }
    }
}

impl From<u8> for OpCode {
    #[inline]
    fn from(value: u8) -> Self {
        Self::from_code(value)
    }
}

impl From<OpCode> for u8 {
    #[inline]
    fn from(value: OpCode) -> Self {
        value.to_code()
    }
}
