use crate::{
    collections::HashTable,
    dbg_println,
    memory::Dealloc,
    value::{
        Value,
        object::{InternedString, ObjectManager},
    },
};

pub struct VmHeap {
    pub objects: ObjectManager,
    pub interned_strings: HashTable<InternedString, ()>,
    pub globals: HashTable<InternedString, Value>,
}

impl VmHeap {
    fn dealloc_interned_strings(&mut self) {
        for entry in self.interned_strings.entries_mut() {
            // Deallocate the key of each entry in the interned strings table
            entry.key.dealloc();
        }
    }
}

impl Drop for VmHeap {
    fn drop(&mut self) {
        dbg_println!("DEBUG DROP VMHEAP");
        self.dealloc_interned_strings();
    }
}

impl std::fmt::Debug for VmHeap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("==== VM HEAP ====\n"))?;
        f.write_fmt(format_args!(
            "Objects: {:?}\n",
            self.objects.get_objects_head()
        ))?;
        f.write_fmt(format_args!(
            "Interned Strings: {:?}\n",
            self.interned_strings
        ))?;
        f.write_fmt(format_args!("=================\n"))?;
        Ok(())
    }
}
