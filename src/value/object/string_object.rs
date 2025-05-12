use std::fmt::Display;

use crate::{
    collections::DynArray,
    dbg_println,
    hashing::{GlobalHashMethod, HashMethod},
    memory::{Dealloc, HeapPtr},
    vm::VmHeap,
};

#[derive(Debug)]
pub struct StringObject {
    char_buf: HeapPtr<DynArray<u8>>,
    hash: u32,
}

impl StringObject {
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

    fn new_raw(chars: DynArray<u8>, heap: &mut VmHeap) -> Self {
        if let Some(entry) = heap
            .interned_strings
            .get::<GlobalHashMethod>(chars.as_str())
        {
            entry.key.clone()
        } else {
            let string = Self {
                hash: GlobalHashMethod::hash(chars.as_slice()),
                char_buf: HeapPtr::alloc(chars),
            };
            heap.interned_strings.insert(string.clone(), None); // We just care about the key.
            string
        }
    }

    pub fn new(str: &str, heap: &mut VmHeap) -> Self {
        if let Some(entry) = heap.interned_strings.get::<GlobalHashMethod>(str) {
            entry.key.clone()
        } else {
            let string = Self {
                hash: GlobalHashMethod::hash(str.as_bytes()),
                char_buf: HeapPtr::alloc(DynArray::from_str(str)),
            };
            heap.interned_strings.insert(string.clone(), None); // We just care about the key.
            string
        }
    }

    pub fn concat(&self, other: &StringObject, heap: &mut VmHeap) -> Self {
        let mut new_char_buf =
            DynArray::new_with_cap(self.char_buf.get_count() + other.char_buf.get_count(), None);
        new_char_buf.push_array(&self.char_buf);
        new_char_buf.push_array(&other.char_buf);
        Self::new_raw(new_char_buf, heap)
    }
}

impl Clone for StringObject {
    /// COPIES OF THE STRING WILL POINT TO THE SAME MEMORY
    fn clone(&self) -> Self {
        Self {
            char_buf: self.char_buf.clone(),
            hash: self.hash,
        }
    }
}

impl Drop for StringObject {
    fn drop(&mut self) {
        dbg_println!("DEBUG RAWSTRING DROP: {}", self.get_str());
        self.char_buf.dealloc();
    }
}

impl PartialEq for StringObject {
    fn eq(&self, other: &Self) -> bool {
        // Since all strings are interned, we can just compare the pointers
        self.char_buf.compare_address(&other.char_buf)
    }
}

impl Display for StringObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.get_str())
    }
}
