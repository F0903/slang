use std::fmt::Display;

use crate::{
    collections::DynArray,
    hashing::{GlobalHashMethod, HashMethod, Hashable},
    value::object::ObjectRef,
};

#[derive(Debug)]
pub struct InternedString {
    char_buf: DynArray<u8>,
    hash: u32,
}

impl InternedString {
    pub(super) fn new_raw(chars: DynArray<u8>) -> Self {
        let hash = GlobalHashMethod::hash(chars.as_slice());
        Self {
            hash,
            char_buf: chars,
        }
    }

    pub(super) fn new(str: &str) -> Self {
        let hash = GlobalHashMethod::hash(str.as_bytes());
        Self {
            hash,
            char_buf: DynArray::from_str(str),
        }
    }

    #[inline]
    pub(super) fn get_char_buf(&self) -> &DynArray<u8> {
        &self.char_buf
    }

    #[inline]
    pub const fn as_slice(&self) -> &[u8] {
        self.char_buf.as_slice()
    }

    #[inline]
    pub const fn as_str(&self) -> &str {
        self.char_buf.as_str()
    }

    #[inline]
    pub const fn get_len(&self) -> usize {
        self.char_buf.get_count()
    }

    #[inline]
    fn compare_contents(&self, other: &InternedString) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl PartialEq for InternedString {
    fn eq(&self, other: &Self) -> bool {
        (self.hash == other.hash) && self.compare_contents(other)
    }
}

impl Display for InternedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Hashable for InternedString {
    fn get_hash(&self) -> u32 {
        self.hash
    }
}

impl Hashable for ObjectRef<InternedString> {
    fn get_hash(&self) -> u32 {
        self.hash
    }
}
