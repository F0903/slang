use super::span::Span;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    MinusEqual,
    Plus,
    PlusEqual,
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
    None,
    Or,
    Return,
    Super,
    This,
    True,
    Let,
    While,
    Continue,
    Break,
    EOF,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Span,
    pub line: u32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: Span, line: u32) -> Self {
        Self {
            token_type,
            lexeme,
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
    __COUNT__,
}

impl Precedence {
    #[inline]
    pub fn add(self, num: usize) -> Precedence {
        debug_assert!(
            num < Precedence::__COUNT__ as usize,
            "'{}' larger than Prececence enum count '{}'!",
            num,
            Precedence::__COUNT__ as usize
        );
        unsafe { std::mem::transmute(self as usize + num) }
    }
}
