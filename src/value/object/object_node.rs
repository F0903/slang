use std::fmt::Debug;

use super::{InternedString, Object, ObjectManager};
use crate::{
    dbg_println,
    memory::{Dealloc, HeapPtr},
    vm::VmHeap,
};
/// A container for objects that "links" them together as a linked list.
/// This is used to keep track of all objects in the VM.
#[derive(Clone, Copy, Debug)]
pub struct ObjectNode {
    obj: HeapPtr<Object>,
    next: HeapPtr<ObjectNode>,
}

impl ObjectNode {
    pub fn alloc(obj: Object, objects: &mut ObjectManager) -> HeapPtr<Self> {
        dbg_println!("DEBUG OBJECT ALLOC: {:?}", obj);

        let head = objects.get_objects_head();

        // Having an additional alloc here for this container type (which is essentially just two pointers) is not optimal, but rust does not allow recursive types without indirection (a pointer) so it cannot be avoided.
        let me = HeapPtr::alloc(Self {
            obj: HeapPtr::alloc(obj),
            next: head,
        });
        objects.set_objects_head(me.clone());
        me
    }

    // Convinience function to allocate a string object
    pub fn alloc_string(str: &str, heap: &mut VmHeap) -> HeapPtr<Self> {
        Self::alloc(
            Object::String(InternedString::new(str, heap)),
            &mut heap.objects,
        )
    }

    pub const fn get_object_ptr(&self) -> HeapPtr<Object> {
        self.obj
    }

    pub fn get_object(&self) -> &Object {
        self.obj.get()
    }

    pub const fn get_next_object_ptr(&self) -> HeapPtr<ObjectNode> {
        self.next
    }
}

impl Dealloc for ObjectNode {
    fn dealloc(&mut self) {
        dbg_println!("DEBUG OBJECT CONTAINER DEALLOC: {:?}", self);
        // Don't dealloc the next node
        self.obj.dealloc();
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
