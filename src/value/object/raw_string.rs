use std::fmt::Display;

use crate::{
    collections::DynArray,
    hashing::{FNV1a, HashMethod},
};

#[derive(Debug)]
pub struct RawString<H: HashMethod = FNV1a> {
    char_buf: DynArray<u8>,
    hash: u32,
    _hash_method: std::marker::PhantomData<H>,
}

impl<H: HashMethod> RawString<H> {
    pub const fn as_slice(&self) -> &[u8] {
        self.char_buf.as_slice()
    }

    pub const fn get_str(&self) -> &str {
        self.char_buf.as_str()
    }

    pub const fn get_len(&self) -> usize {
        self.char_buf.get_count()
    }

    pub const fn get_hash(&self) -> u32 {
        self.hash
    }

    fn new_raw(chars: DynArray<u8>) -> Self {
        Self {
            hash: H::hash(chars.as_slice()),
            char_buf: chars,
            _hash_method: std::marker::PhantomData,
        }
    }

    pub fn new(str: &str) -> Self {
        Self {
            hash: H::hash(str.as_bytes()),
            char_buf: DynArray::from_str(str),
            _hash_method: std::marker::PhantomData,
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

impl<H: HashMethod> Drop for RawString<H> {
    fn drop(&mut self) {
        println!("DEBUG RAWSTRING DROP: {}", self.get_str());
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
