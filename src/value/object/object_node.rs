use std::fmt::Debug;

use super::{InternedString, Object};
use crate::{
    dbg_println,
    memory::{Dealloc, HeapPtr},
    vm::VmHeap,
};

/// A container for objects that "links" them together as a linked list.
/// This is used to keep track of all objects in the VM.
#[derive(Clone, Debug)]
pub struct ObjectNode {
    obj: Object,
    next: HeapPtr<ObjectNode>,
}

impl ObjectNode {
    pub fn alloc(obj: Object, heap: &mut VmHeap) -> HeapPtr<Self> {
        dbg_println!("DEBUG OBJECT ALLOC: {:?}", obj);

        let head = heap.get_objects_head();

        let me = HeapPtr::alloc(Self { obj, next: head });
        heap.set_objects_head(me.clone());
        me
    }

    // Convinience function to allocate a string object
    pub fn alloc_string(str: &str, heap: &mut VmHeap) -> HeapPtr<Self> {
        Self::alloc(Object::String(InternedString::new(str, heap)), heap)
    }

    pub fn get_object(&self) -> &Object {
        &self.obj
    }

    pub const fn get_next_object_ptr(&self) -> HeapPtr<ObjectNode> {
        self.next
    }
}

impl Dealloc for ObjectNode {
    fn dealloc(&mut self) {
        dbg_println!("DEBUG OBJECTNODE DEALLOC: {:?}", self);
        self.obj.dealloc();

        // We don't deallocate the next node here, as we want the rest of the objects to remain.
    }
}

impl PartialEq for ObjectNode {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}

impl PartialOrd for ObjectNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            _ => {
                if self == other {
                    Some(std::cmp::Ordering::Equal)
                } else {
                    None
                }
            }
        }
    }
}
