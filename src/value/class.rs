use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
}

impl Class {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}
