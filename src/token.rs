use crate::identifiable::Identifiable;

pub struct VariableInfo {
    indentifier: &'static str,
}

pub struct FunctionInfo {
    start_ident: &'static str,
    end_ident: &'static str,
}

pub enum Token {
    Variable(VariableInfo),
    Function(FunctionInfo),
}

const TOKENS: &[Token] = &[
    Token::Variable(VariableInfo {
        indentifier: "offering",
    }),
    Token::Function(FunctionInfo {
        start_ident: "ritual",
        end_ident: "end",
    }),
];

impl Identifiable for Token {
    fn get_identifier(&self) -> &'static str {
        match self {
            Token::Variable(x) => x.indentifier,
            Token::Function(x) => x.start_ident,
        }
    }
}

pub struct TokenInstance<'a> {
    pub index: usize,
    pub token: &'a Token,
}

pub fn get_tokens(line: &str) -> Vec<TokenInstance> {
    let mut token_buf = vec![];
    for token in TOKENS {
        let indent: &dyn Identifiable = token;

        match line.find(indent.get_identifier()) {
            Some(i) => token_buf.push(TokenInstance { token, index: i }),
            None => continue,
        };
    }
    token_buf
}
