use std::mem::ManuallyDrop;

use crate::{
    collections::{DynArray, HashTable},
    hashing::GlobalHashMethod,
    memory::{Dealloc, GC},
    value::{
        ObjectType,
        object::{ObjectRef, ObjectUnion, String},
    },
};

#[derive(Debug)]
pub struct StringInterner {
    strings: HashTable<ObjectRef<String>, ()>,
}

impl StringInterner {
    pub const fn new() -> Self {
        Self {
            strings: HashTable::new(),
        }
    }

    fn create_string(&mut self, str: &str) -> ObjectRef<String> {
        let string = String::new(str);
        let string_object = GC
            .create_object(
                ObjectType::String,
                ObjectUnion {
                    string: ManuallyDrop::new(string),
                },
            )
            .as_string();
        self.strings.set(string_object, ());
        string_object
    }

    fn create_string_raw(&mut self, chars: DynArray<u8>) -> ObjectRef<String> {
        let string = String::new_raw(chars);
        let string_object = GC
            .create_object(
                ObjectType::String,
                ObjectUnion {
                    string: ManuallyDrop::new(string),
                },
            )
            .as_string();
        self.strings.set(string_object, ());
        string_object
    }

    pub fn get_interned_strings_count(&self) -> usize {
        self.strings.count()
    }

    pub(crate) fn get_interned_strings(&self) -> impl Iterator<Item = ObjectRef<String>> {
        self.strings.entries().map(|x| x.key)
    }

    pub fn remove(&mut self, string: ObjectRef<String>) -> Result<(), &'static str> {
        let string = self
            .strings
            .delete(string)
            .map(|x| x.key)
            .ok_or("Could not find string to remove!")?;
        string.upcast().dealloc();
        Ok(())
    }

    pub fn make_string(&mut self, str: &str) -> ObjectRef<String> {
        self.strings
            .get_by_str::<GlobalHashMethod>(str)
            .map(|x| x.key)
            .unwrap_or_else(|| self.create_string(str))
    }

    pub fn concat_strings(
        &mut self,
        lhs: ObjectRef<String>,
        rhs: ObjectRef<String>,
    ) -> ObjectRef<String> {
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
            entry.key.upcast().dealloc();
        }

        for hash in hashes_to_delete {
            self.strings.delete_by_hash(hash);
        }
    }
}
