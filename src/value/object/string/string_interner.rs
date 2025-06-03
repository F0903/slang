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
    pub fn new() -> Self {
        Self {
            strings: HashTable::new(),
        }
    }

    fn create_string(&mut self, str: &str) -> InternedString {
        let string = InternedString::new(str);
        self.strings.set(string.clone(), ());
        string
    }

    pub fn make_string(&mut self, str: &str) -> InternedString {
        self.strings
            .get_by_str::<GlobalHashMethod>(str)
            .map(|x| x.key)
            .unwrap_or_else(|| self.create_string(str))
    }

    pub fn concat_strings(&self, a: InternedString, b: InternedString) -> InternedString {
        let mut new_char_buf = DynArray::new_with_cap(a.get_len() + b.get_len());
        new_char_buf.push_array(a.get_char_buf());
        new_char_buf.push_array(b.get_char_buf());
        InternedString::new_raw(new_char_buf)
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
