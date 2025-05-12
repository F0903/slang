use std::fmt::Display;

use crate::{
    collections::DynArray,
    hashing::{FNV1a, HashMethod},
    memory::{Dealloc, ManualPtr},
};

#[derive(Debug)]
pub struct StringObject<H: HashMethod = FNV1a> {
    char_buf: ManualPtr<DynArray<u8>>,
    hash: u32,
    _hash_method: std::marker::PhantomData<H>,
}

impl<H: HashMethod> StringObject<H> {
    pub const fn as_slice(&self) -> &[u8] {
        self.char_buf.get().as_slice()
    }

    pub const fn get_str(&self) -> &str {
        self.char_buf.get().as_str()
    }

    pub const fn get_len(&self) -> usize {
        self.char_buf.get().get_count()
    }

    pub const fn get_hash(&self) -> u32 {
        self.hash
    }

    fn new_raw(chars: DynArray<u8>) -> Self {
        Self {
            hash: H::hash(chars.as_slice()),
            char_buf: ManualPtr::alloc(chars),
            _hash_method: std::marker::PhantomData,
        }
    }

    pub fn new(str: &str) -> Self {
        Self {
            hash: H::hash(str.as_bytes()),
            char_buf: ManualPtr::alloc(DynArray::from_str(str)),
            _hash_method: std::marker::PhantomData,
        }
    }

    pub fn concat(&self, other: &StringObject) -> Self {
        let mut new_char_buf =
            DynArray::new_with_cap(self.char_buf.get_count() + other.char_buf.get_count(), None);
        new_char_buf.push_array(&self.char_buf);
        new_char_buf.push_array(&other.char_buf);
        Self::new_raw(new_char_buf)
    }
}

impl Clone for StringObject {
    /// COPIES OF THE STRING WILL POINT TO THE SAME MEMORY
    fn clone(&self) -> Self {
        Self {
            char_buf: self.char_buf.clone(),
            hash: self.hash,
            _hash_method: self._hash_method,
        }
    }
}

impl<H: HashMethod> Drop for StringObject<H> {
    fn drop(&mut self) {
        println!("DEBUG RAWSTRING DROP: {}", self.get_str());
        self.char_buf.dealloc();
    }
}

impl PartialEq for StringObject {
    fn eq(&self, other: &Self) -> bool {
        let me = self.get_str();
        let other = other.get_str();
        me == other
    }
}

impl Display for StringObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.get_str())
    }
}
