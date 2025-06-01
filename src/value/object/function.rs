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
}

impl Function {
    pub fn new(arity: u8, chunk: HeapPtr<Chunk>, name: Option<InternedString>) -> Self {
        Self { arity, chunk, name }
    }

    pub fn set_name(&mut self, name: Option<InternedString>) {
        self.name = name;
    }
}

impl Dealloc for Function {
    fn dealloc(&mut self) {
        dbg_println!("DEBUG FUNCTION DEALLOC: {:?}", self);
        if !self.chunk.is_null() {
            self.chunk.dealloc();
            self.chunk = HeapPtr::null();
        }

        if let Some(name) = &mut self.name {
            name.dealloc();
            self.name = None;
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity && self.chunk.compare_address(&other.chunk)
    }
}
