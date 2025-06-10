use crate::{memory::GcPtr, value::Object};

pub trait AsObjectPtr {
    fn as_object_ptr(&self) -> GcPtr<Object>;
}
