use super::InternedString;
use crate::{
    compiler::chunk::Chunk,
    memory::{Dealloc, HeapPtr},
};

#[derive(Clone, Debug)]
pub struct Function {
    pub arity: u8,
    pub chunk: HeapPtr<Chunk>,
    name: Option<InternedString>,
}

impl Function {
    pub fn new(arity: u8, chunk: HeapPtr<Chunk>, name: Option<InternedString>) -> Self {
        Self { arity, chunk, name }
    }

    pub fn get_name(&self) -> &Option<InternedString> {
        &self.name
    }

    pub fn set_name(&mut self, name: Option<InternedString>) {
        self.name = name;
    }
}

impl Dealloc for Function {
    fn dealloc(&mut self) {
        if !self.chunk.is_null() {
            self.chunk.dealloc();
        }
        if let Some(name) = &mut self.name {
            name.dealloc();
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity && self.chunk.compare_address(&other.chunk)
    }
}
