use std::iter;

use crate::{
    collections::{DynArray, HashTable},
    hashing::GlobalHashMethod,
    value::object::InternedString,
};

#[derive(Debug)]
pub struct StringInterner {
    strings: HashTable<InternedString, ()>,
}

impl StringInterner {
    pub const fn new() -> Self {
        Self {
            strings: HashTable::new(),
        }
    }

    fn create_string(&mut self, str: &str) -> InternedString {
        let string = InternedString::new(str);
        self.strings.set(string, ());
        string
    }

    fn create_string_raw(&mut self, chars: DynArray<u8>) -> InternedString {
        let string = InternedString::new_raw(chars);
        self.strings.set(string, ());
        string
    }

    pub fn get_interned_strings_count(&self) -> usize {
        self.strings.count()
    }

    pub(crate) fn get_interned_strings(&self) -> impl Iterator<Item = InternedString> {
        self.strings.entries().map(|x| x.key)
    }

    pub fn remove(&mut self, string: InternedString) -> Result<(), &'static str> {
        let mut string = self
            .strings
            .delete(&string)
            .map(|x| x.key)
            .ok_or("Could not find string to remove!")?;
        string.dealloc();
        Ok(())
    }

    pub fn make_string(&mut self, str: &str) -> InternedString {
        self.strings
            .get_by_str::<GlobalHashMethod>(str)
            .map(|x| x.key)
            .unwrap_or_else(|| self.create_string(str))
    }

    pub fn concat_strings(&mut self, lhs: InternedString, rhs: InternedString) -> InternedString {
        let mut new_char_buf = DynArray::new_with_cap(lhs.get_len() + rhs.get_len());
        new_char_buf.push_array(lhs.get_char_buf());
        new_char_buf.push_array(rhs.get_char_buf());
        self.create_string_raw(new_char_buf)
    }
}

impl Drop for StringInterner {
    fn drop(&mut self) {
        let mut hashes_to_delete = DynArray::new_with_cap(self.strings.count());
        for entry in self.strings.entries_mut() {
            // Deallocate the key of each entry in the interned strings table
            hashes_to_delete.push(entry.key.get_hash());
            entry.key.dealloc();
        }

        for hash in hashes_to_delete {
            self.strings.delete_by_hash(hash);
        }
    }
}
