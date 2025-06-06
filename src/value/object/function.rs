use std::fmt::Display;

use super::InternedString;
use crate::{compiler::chunk::Chunk, dbg_println};

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: u8,
    pub chunk: Chunk,
    pub name: Option<InternedString>,
    pub upvalue_count: u16,
}

impl Drop for Function {
    fn drop(&mut self) {
        dbg_println!("DEBUG FUNCTION DROP: {:?}", self);
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity && ((self as *const _) == (other as *const _))
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "fn {:?}[{}] ({} upvalues) = {:?}",
            self.name
                .map(|x| x.as_str().to_owned())
                .unwrap_or("<script>".to_owned()),
            self.arity,
            self.upvalue_count,
            self.chunk.get_code_ptr()
        ))
    }
}
