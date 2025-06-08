use std::fmt::Display;

use crate::{compiler::chunk::Chunk, dbg_println, value::object::InternedString};

#[derive(Debug, Clone)]
pub struct Function {
    arity: u8,
    chunk: Chunk,
    name: Option<InternedString>,
    upvalue_count: u16,
}

impl Function {
    pub const fn new(
        arity: u8,
        chunk: Chunk,
        name: Option<InternedString>,
        upvalue_count: u16,
    ) -> Self {
        Self {
            arity,
            chunk,
            name,
            upvalue_count,
        }
    }

    pub const fn get_arity(&self) -> u8 {
        self.arity
    }

    pub fn increment_arity(&mut self, increment: u8) {
        self.arity += increment;
    }

    pub const fn get_chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub const fn get_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.chunk
    }

    pub fn set_name(&mut self, name: InternedString) {
        self.name = Some(name);
    }

    pub fn get_name(&self) -> Option<InternedString> {
        self.name
    }

    pub fn get_upvalue_count(&self) -> u16 {
        self.upvalue_count
    }

    pub fn increment_upvalue_count(&mut self, increment: u16) {
        self.upvalue_count += increment;
    }
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
            self.get_name()
                .map(|x| x.as_str().to_owned())
                .unwrap_or("<script>".to_owned()),
            self.arity,
            self.upvalue_count,
            self.chunk.get_code_ptr()
        ))
    }
}
