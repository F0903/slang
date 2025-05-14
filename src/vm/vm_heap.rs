use crate::{
    collections::HashTable,
    dbg_println,
    value::{
        Value,
        object::{ObjectManager, StringObject},
    },
};

pub struct VmHeap {
    pub objects: ObjectManager,
    pub interned_strings: HashTable<StringObject>,
    pub globals: HashTable<Value>,
}

impl Drop for VmHeap {
    fn drop(&mut self) {
        dbg_println!("DEBUG DROP VMHEAP");
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
