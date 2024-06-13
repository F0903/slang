mod object_ptr;
mod raw_string;

pub use object_ptr::ObjectPtr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectType {
    RawString,
}

pub struct Object {
    obj_type: ObjectType,
}
