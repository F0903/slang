use std::fmt::Display;

use super::InternedString;
use crate::{
    compiler::chunk::Chunk,
    dbg_println,
    memory::{Dealloc, HeapPtr},
};

#[derive(Clone, Debug)]
pub struct Function {
    pub arity: u8,
    pub chunk: HeapPtr<Chunk>,
    pub name: Option<InternedString>,
    pub upvalue_count: u16,
}

impl Dealloc for Function {
    fn dealloc(&mut self) {
        dbg_println!("DEBUG FUNCTION DEALLOC: {:?}", self);
        if !self.chunk.is_null() {
            self.chunk.dealloc();
            self.chunk = HeapPtr::null();
        }

        // Name is an interned string, so we don't deallocate it here.
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity && self.chunk.compare_address(&other.chunk)
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
