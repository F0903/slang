use crate::lexing::token::Token;

#[derive(Debug)]
pub struct Local {
    pub name: Token,
    pub depth: i32,
}

impl Local {
    pub fn new(name: Token, depth: i32) -> Self {
        Self { name, depth }
    }

    pub fn initialize(&mut self, depth: i32) {
        self.depth = depth;
    }

    pub fn is_initialized(&self) -> bool {
        // Uninitialized values have a depth of -1 (used instead of a field to save memory)
        self.depth > 0
    }
}
