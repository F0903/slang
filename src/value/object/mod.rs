mod raw_string;

pub use raw_string::RawString;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectType {
    String,
}

#[repr(C)]
pub struct Object {
    obj_type: ObjectType,
    next: *mut Object,
}

impl Object {
    pub const fn get_type(&self) -> ObjectType {
        self.obj_type
    }

    pub const fn get_next_object(&self) -> *mut Object {
        self.next
    }
}
