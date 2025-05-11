#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Equal,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    For,
    Fn,
    If,
    Is,
    Not,
    IsNot,
    None,
    Or,
    Return,
    Super,
    This,
    True,
    Let,
    While,
    EOF,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub name: String,
    pub line: u32,
}

impl Token {
    pub fn new(typ: TokenType, name: impl Into<String>, line: u32) -> Self {
        Self {
            token_type: typ,
            name: name.into(),
            line,
        }
    }
}

// Order is important!
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
#[repr(usize)]
pub enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // is
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // not -
    Call,       // . ()
    Primary,
}

impl Precedence {
    pub fn add(self, num: usize) -> Precedence {
        unsafe { std::mem::transmute(self as usize + num) }
    }
}
