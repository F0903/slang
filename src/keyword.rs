pub trait KeywordInfo {
    fn get_keyword(&self) -> &'static str;
}

pub enum Keyword {
    Variable(&'static str),
    Function(&'static str),
    IfScope(&'static str),
    RepeatScope(&'static str),
    ScopeBreak(&'static str),
    ScopeEnd(&'static str),
}

pub const KEYWORDS: &[Keyword] = &[
    Keyword::Variable("offering"),
    Keyword::Function("ritual"),
    Keyword::IfScope("if"),
    Keyword::RepeatScope("repeat"),
    Keyword::ScopeBreak("break"),
    Keyword::ScopeEnd("end"),
];

impl KeywordInfo for Keyword {
    fn get_keyword(&self) -> &'static str {
        match self {
            Keyword::Variable(x) => x,
            Keyword::Function(x) => x,
            Keyword::IfScope(x) => x,
            Keyword::RepeatScope(x) => x,
            Keyword::ScopeBreak(x) => x,
            Keyword::ScopeEnd(x) => x,
        }
    }
}
