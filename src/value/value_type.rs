use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ValueType {
    Bool,
    None,
    Number,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Bool => "Bool",
            Self::Number => "Number",
            Self::None => "None",
        })
    }
}
