use std::fmt::Debug;

use crate::{
    collections::HashTable,
    dbg_println,
    memory::{Dealloc, HeapPtr},
    value::{
        Value,
        object::{InternedString, Object, StringInterner},
    },
};

pub struct VmHeap {
    pub objects_head: HeapPtr<Object>,
    pub globals: HashTable<InternedString, Value>,
    pub strings: StringInterner,
}

impl VmHeap {
    pub fn get_objects_head(&self) -> HeapPtr<Object> {
        self.objects_head
    }

    pub fn set_objects_head(&mut self, head: HeapPtr<Object>) {
        // We do not deallocate the old head here, as we are building a linked list, which is set internally in each node.
        self.objects_head = head;
    }

    pub fn print_state(&self) {
        dbg_println!("==== VM HEAP ====\n");
        dbg_println!("Objects: {:?}\n", self.objects_head);
        dbg_println!("Interned Strings: {:?}\n", self.strings);
        dbg_println!("=================\n");
    }
}

impl Debug for VmHeap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // We can't call self.print_state here, since it is possible that self.objects or self.interned_strings have been deallocated.
        f.write_str("VmHeap")
    }
}
