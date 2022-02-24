use crate::util::LINE_ENDING;

pub struct VariableInfo {
    start_token: &'static str,
    end_token: &'static str,
}

pub struct FunctionInfo {
    start_token: &'static str,
    end_token: &'static str,
}

pub trait TokenInfo {
    fn get_start_token(&self) -> &'static str;
    fn get_end_token(&self) -> &'static str;
}

pub enum Token {
    Variable(VariableInfo),
    Function(FunctionInfo),
}

pub const TOKENS: &[Token] = &[
    Token::Variable(VariableInfo {
        start_token: "offering",
        end_token: LINE_ENDING,
    }),
    Token::Function(FunctionInfo {
        start_token: "ritual",
        end_token: "end",
    }),
];

impl TokenInfo for Token {
    fn get_start_token(&self) -> &'static str {
        match self {
            Token::Variable(x) => x.start_token,
            Token::Function(x) => x.start_token,
        }
    }

    fn get_end_token(&self) -> &'static str {
        match self {
            Token::Variable(x) => x.end_token,
            Token::Function(x) => x.end_token,
        }
    }
}

pub struct TokenInstance<'a> {
    pub start_index: usize,
    pub end_index: usize,
    pub token: &'a Token,
}
