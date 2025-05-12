use crate::{memory::HeapPtr, value::ObjectContainer};

#[derive(Debug)]
pub struct ObjectManager {
    objects_head: HeapPtr<ObjectContainer>,
}

impl ObjectManager {
    pub const fn new() -> Self {
        Self {
            objects_head: HeapPtr::null(),
        }
    }

    pub const fn get_objects_head(&self) -> HeapPtr<ObjectContainer> {
        self.objects_head
    }

    pub const fn set_objects_head(&mut self, object: HeapPtr<ObjectContainer>) {
        self.objects_head = object;
    }
}
