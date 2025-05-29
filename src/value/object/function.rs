use super::InternedString;
use crate::{
    compiler::chunk::Chunk,
    memory::{Dealloc, HeapPtr},
};

#[derive(Clone, Debug)]
pub struct Function {
    pub arity: u32,
    pub chunk: HeapPtr<Chunk>,
    name: InternedString,
}

impl Function {
    pub fn new(arity: u32, chunk: HeapPtr<Chunk>, name: InternedString) -> Self {
        Self { arity, chunk, name }
    }

    pub fn get_name(&self) -> &InternedString {
        &self.name
    }
}

impl Dealloc for Function {
    fn dealloc(&mut self) {
        if !self.chunk.is_null() {
            self.chunk.dealloc();
        }
        self.name.dealloc();
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity && self.chunk.compare_address(&other.chunk)
    }
}
