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
        for bucket in self.interned_strings.get_raw_data().memory_iter() {
            let bucket = unsafe { bucket.assume_init_read() };
            let entry = bucket.entry;
            match entry {
                Some(mut entry) => {
                    entry.key.dealloc();
                }
                None => (),
            }
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
