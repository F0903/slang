use std::fmt::Debug;

use crate::{
    collections::HashTable,
    dbg_println,
    memory::{Dealloc, HeapPtr},
    value::{
        Value,
        object::{InternedString, ObjectNode},
    },
};

pub struct VmHeap {
    pub objects_head: HeapPtr<ObjectNode>,
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

    pub fn get_objects_head(&self) -> HeapPtr<ObjectNode> {
        self.objects_head
    }

    pub fn set_objects_head(&mut self, head: HeapPtr<ObjectNode>) {
        // We do not deallocate the old head here, as we are building a linked list, which is set internally in each node.
        self.objects_head = head;
    }

    pub fn print_state(&self) {
        dbg_println!("==== VM HEAP ====\n");
        dbg_println!("Objects: {:?}\n", self.objects_head);
        dbg_println!("Interned Strings: {:?}\n", self.interned_strings);
        dbg_println!("=================\n");
    }
}

impl Dealloc for VmHeap {
    fn dealloc(&mut self) {
        dbg_println!("DEBUG DROP VMHEAP");
        self.dealloc_interned_strings();
    }
}

impl Debug for VmHeap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // We can't call self.print_state here, since it is possible that self.objects or self.interned_strings have been deallocated.
        f.write_str("VmHeap")
    }
}
