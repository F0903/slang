use crate::lexing::token::Token;

#[derive(Debug)]
pub struct Local {
    name: Option<Token>,
    depth: i32,
    is_captured: bool,
}

impl Local {
    pub const fn new(name: Token, depth: i32) -> Self {
        Self {
            name: Some(name),
            depth,
            is_captured: false,
        }
    }

    /// Creates a dummy local variable.
    /// Primarily used for reserving stack slots.
    pub const fn dummy() -> Self {
        Self {
            name: None,
            depth: -1, // Uninitialized
            is_captured: false,
        }
    }

    pub const fn get_name(&self) -> Option<&Token> {
        self.name.as_ref()
    }

    pub const fn get_depth(&self) -> i32 {
        self.depth
    }

    pub const fn capture(&mut self) {
        self.is_captured = true;
    }

    pub const fn is_captured(&self) -> bool {
        self.is_captured
    }

    pub const fn initialize(&mut self, depth: i32) {
        self.depth = depth;
    }

    pub const fn is_initialized(&self) -> bool {
        // Uninitialized values have a depth of -1 (used instead of a field to save memory)
        self.depth > 0
    }
}
