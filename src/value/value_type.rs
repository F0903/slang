use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ValueType {
    Bool,
    None,
    Number,
    Object,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Bool => "Bool",
            Self::Number => "Number",
            Self::Object => "Object",
            Self::None => "None",
        })
    }
}
