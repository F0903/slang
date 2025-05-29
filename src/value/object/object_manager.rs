use crate::{memory::HeapPtr, value::ObjectNode};

#[derive(Debug)]
pub struct ObjectManager {
    objects_head: HeapPtr<ObjectNode>,
}

impl ObjectManager {
    pub const fn new() -> Self {
        Self {
            objects_head: HeapPtr::null(),
        }
    }

    pub const fn get_objects_head(&self) -> HeapPtr<ObjectNode> {
        self.objects_head
    }

    pub const fn set_objects_head(&mut self, object: HeapPtr<ObjectNode>) {
        self.objects_head = object;
    }
}
