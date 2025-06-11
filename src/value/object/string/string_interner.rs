use crate::{
    collections::{DynArray, HashTable},
    hashing::GlobalHashMethod,
    memory::{GC, GcScopedRoot},
    value::object::{InternedString, ObjectRef},
};

#[derive(Debug)]
pub struct StringInterner {
    // Since the objects lifetimes are managed by the GC, we can't deallocate them here.
    strings: HashTable<ObjectRef<InternedString>, ()>,
}

impl StringInterner {
    pub const fn new() -> Self {
        Self {
            strings: HashTable::new(),
        }
    }

    pub fn get_interned_strings_count(&self) -> usize {
        self.strings.count()
    }

    pub(crate) fn get_interned_strings(&self) -> impl Iterator<Item = ObjectRef<InternedString>> {
        self.strings.entries().map(|x| x.key)
    }

    pub fn remove(
        &mut self,
        string: ObjectRef<InternedString>,
    ) -> Result<ObjectRef<InternedString>, &'static str> {
        let string = self
            .strings
            .delete(string)
            .map(|x| x.key)
            .ok_or("Could not find string to remove!")?;
        Ok(string)
    }

    fn create_string(&mut self, str: &str) -> GcScopedRoot<ObjectRef<InternedString>> {
        let new_string = GC.create_string(InternedString::new(str));
        self.strings.set(new_string.get_object(), ());
        new_string
    }

    pub fn make_string(&mut self, str: &str) -> GcScopedRoot<ObjectRef<InternedString>> {
        self.strings
            .get_by_str::<GlobalHashMethod>(str)
            .map(|x| x.key)
            .map(|string| GcScopedRoot::register(string))
            .unwrap_or_else(|| self.create_string(str))
    }

    pub fn concat_strings(
        &mut self,
        lhs: ObjectRef<InternedString>,
        rhs: ObjectRef<InternedString>,
    ) -> GcScopedRoot<ObjectRef<InternedString>> {
        let mut new_char_buf = DynArray::new_with_cap(lhs.get_len() + rhs.get_len());
        new_char_buf.push_array(lhs.get_char_buf());
        new_char_buf.push_array(rhs.get_char_buf());
        GC.create_string(InternedString::new_raw(new_char_buf))
    }
}
