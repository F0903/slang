use std::collections::HashMap;

macro_rules! hashmap {
    ($($key:expr => $val:expr),*) => {{
        let mut hmap = ::std::collections::HashMap::new();
        $(hmap.insert($key, $val);)*
        hmap
    }};
}

pub trait KeywordInfo {
    fn get_keyword(&self) -> &'static str;
}

#[derive(Eq, PartialEq, Hash)]
pub enum Keyword {
    Variable,
    Function,
    IfScope,
    RepeatScope,
    ScopeBreak,
    ScopeEnd,
    ScopeReturn,
}

lazy_static! {
    pub static ref KEYWORDS: HashMap<Keyword, &'static str> = hashmap! {
        Keyword::Variable => "offering",
        Keyword::Function => "ritual",
        Keyword::IfScope => "if",
        Keyword::RepeatScope => "repeat",
        Keyword::ScopeBreak => "break",
        Keyword::ScopeEnd => "end",
        Keyword::ScopeReturn => "return"
    };
}
impl KeywordInfo for Keyword {
    #[inline]
    fn get_keyword(&self) -> &'static str {
        KEYWORDS[self]
    }
}
