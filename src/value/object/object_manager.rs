use crate::{memory::ManualPtr, value::ObjectContainer};

pub struct ObjectManager {
    objects_head: ManualPtr<ObjectContainer>,
}

impl ObjectManager {
    pub const fn new() -> Self {
        Self {
            objects_head: ManualPtr::null(),
        }
    }

    pub const fn get_objects_head(&self) -> ManualPtr<ObjectContainer> {
        self.objects_head
    }

    pub const fn set_objects_head(&mut self, object: ManualPtr<ObjectContainer>) {
        self.objects_head = object;
    }
}
