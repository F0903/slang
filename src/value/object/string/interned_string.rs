use std::{fmt::Display, ops::Add};

use crate::{
    collections::DynArray,
    dbg_println,
    hashing::{GlobalHashMethod, HashMethod, Hashable},
    memory::{Dealloc, GC, HeapPtr},
    value::object::ObjectRef,
};

// Since this is essentially just a wrapper around a pointer with a hash, we can copy it very cheaply
#[derive(Debug, Clone, Copy)]
pub struct InternedString {
    char_buf: HeapPtr<DynArray<u8>>,
    hash: u32,
    // Since we treat strings as a Value instead of an Object (thus saving a pointer hop),
    // we implement the Object GC marking here as well.
    marked: bool,
}

impl InternedString {
    pub(super) fn new_raw(chars: DynArray<u8>) -> Self {
        Self {
            hash: GlobalHashMethod::hash(chars.as_slice()),
            char_buf: HeapPtr::alloc(chars),
            marked: false,
        }
    }

    pub(super) fn new(str: &str) -> Self {
        Self {
            hash: GlobalHashMethod::hash(str.as_bytes()),
            char_buf: HeapPtr::alloc(DynArray::from_str(str)),
            marked: false,
        }
    }

    #[inline]
    pub(crate) const fn is_marked(&self) -> bool {
        self.marked
    }

    #[inline]
    pub(crate) const fn mark(&mut self) {
        self.marked = true;
    }

    #[inline]
    pub(crate) const fn unmark(&mut self) {
        self.marked = false;
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
        self.char_buf.get().as_slice()
    }

    #[inline]
    pub const fn as_str(&self) -> &str {
        self.char_buf.get().as_str()
    }

    #[inline]
    pub const fn get_len(&self) -> usize {
        self.char_buf.get().get_count()
    }

    #[inline]
    pub const fn get_hash(&self) -> u32 {
        self.hash
    }

    // We put this in the main impl intead of implementing Dealloc, as we only want the StringInterner to be able to dealloc this.
    // We rely on manual dealloc instead of Drop, as these are interned, thereby all potentially sharing memory.
    pub(super) fn dealloc(&mut self) {
        dbg_println!("DEBUG RAWSTRING DEALLOC: {}", self.as_str());
        self.char_buf.dealloc();
    }
}

impl Add for InternedString {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        GC.concat_strings(self, rhs)
    }
}

impl PartialEq for InternedString {
    fn eq(&self, other: &Self) -> bool {
        // Since all strings are interned, we can just compare the pointers
        (self as *const _) == (other as *const _)
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
