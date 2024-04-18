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
    Bang,
    //BangEqual,
    Equal,
    //EqualEqual,
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
    None,
    Not,
    Or,
    Return,
    Super,
    This,
    True,
    Let,
    While,
    EOF,
}

#[derive(Clone)]
pub struct Token {
    typ: TokenType,
    name: String,
    line: u32,
}

impl Token {
    pub fn new(typ: TokenType, name: impl Into<String>, line: u32) -> Self {
        Self {
            typ,
            name: name.into(),
            line,
        }
    }

    pub const fn get_type(&self) -> TokenType {
        self.typ
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub const fn get_line(&self) -> u32 {
        self.line
    }
}

// Order is important!
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(usize)]
pub enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // is not
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    pub fn Add(self, num: usize) -> Precedence {
        unsafe { std::mem::transmute((self as usize + num)) }
    }
}
