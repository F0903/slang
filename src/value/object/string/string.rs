use std::fmt::Display;

use crate::{
    collections::DynArray,
    hashing::{GlobalHashMethod, HashMethod, Hashable},
    value::object::ObjectRef,
};

#[derive(Debug)]
pub struct String {
    char_buf: DynArray<u8>,
    hash: u32,
}

impl String {
    pub(super) fn new_raw(chars: DynArray<u8>) -> Self {
        Self {
            hash: GlobalHashMethod::hash(chars.as_slice()),
            char_buf: chars,
        }
    }

    pub(super) fn new(str: &str) -> Self {
        Self {
            hash: GlobalHashMethod::hash(str.as_bytes()),
            char_buf: DynArray::from_str(str),
        }
    }

    #[inline]
    pub(super) fn get_char_buf(&self) -> &DynArray<u8> {
        &self.char_buf
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.as_str().is_empty()
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
    pub const fn get_hash(&self) -> u32 {
        self.hash
    }
}

impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        // Since all strings are interned, we can just compare the pointers
        (self as *const _) == (other as *const _)
    }
}

impl Display for String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Hashable for String {
    fn get_hash(&self) -> u32 {
        self.hash
    }
}

impl Hashable for ObjectRef<String> {
    fn get_hash(&self) -> u32 {
        self.hash
    }
}
