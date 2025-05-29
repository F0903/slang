use crate::lexing::token::Token;

#[derive(Debug)]
pub struct Local {
    name: Option<Token>,
    depth: i32,
}

impl Local {
    pub fn new(name: Token, depth: i32) -> Self {
        Self {
            name: Some(name),
            depth,
        }
    }

    /// Creates a dummy local variable.
    /// Primarily used for reserving stack slots.
    pub fn dummy() -> Self {
        Self {
            name: None,
            depth: -1, // Uninitialized
        }
    }

    pub fn get_name(&self) -> Option<&Token> {
        self.name.as_ref()
    }

    pub fn get_depth(&self) -> i32 {
        self.depth
    }

    pub fn initialize(&mut self, depth: i32) {
        self.depth = depth;
    }

    pub fn is_initialized(&self) -> bool {
        // Uninitialized values have a depth of -1 (used instead of a field to save memory)
        self.depth > 0
    }
}
