use std::fmt::Display;

use crate::{dynarray::DynArray, memory::Dealloc};

#[derive(Debug)]
pub struct RawString {
    char_buf: DynArray<u8>,
}

impl RawString {
    pub const fn get_str(&self) -> &str {
        self.char_buf.as_str()
    }

    pub const fn get_len(&self) -> usize {
        self.char_buf.get_count()
    }

    fn new_raw(chars: DynArray<u8>) -> Self {
        Self { char_buf: chars }
    }

    pub fn new(str: &str) -> Self {
        Self {
            char_buf: DynArray::from_str(str),
        }
    }

    pub fn concat(&self, other: &RawString) -> Self {
        let mut new_char_buf =
            DynArray::new_with_cap(self.char_buf.get_count() + other.char_buf.get_count());
        new_char_buf.push_array(&self.char_buf);
        new_char_buf.push_array(&other.char_buf);
        Self::new_raw(new_char_buf)
    }
}

impl Dealloc for RawString {
    fn dealloc(&mut self) {
        println!("DEBUG DEALLOC: {}", self);
        self.char_buf.dealloc()
    }
}

impl PartialEq for RawString {
    fn eq(&self, other: &Self) -> bool {
        let me = self.get_str();
        let other = other.get_str();
        me == other
    }
}

impl Display for RawString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.get_str())
    }
}
