use std::ptr::null_mut;

use crate::memory::reallocate;

use super::{Object, ObjectPtr};

pub struct RawString {
    parent: Object,
    len: usize,
    chars: *mut u8, // Heap alloced, will need to be freed.
}

impl RawString {
    pub fn alloc_from_str(value: &str) -> ObjectPtr {
        unsafe {
            let string_ptr = reallocate::<RawString>(null_mut(), 0, 1);
            let obj_ptr = string_ptr.cast::<RawString>();
            let mut obj = obj_ptr.read();
            obj.parent = Object {
                obj_type: super::ObjectType::RawString,
            };

            reallocate::<u8>(obj.chars, 0, value.len());
            value
                .as_ptr()
                .copy_to_nonoverlapping(obj.chars, value.len());
            obj.len = value.len();

            ObjectPtr::new(obj_ptr.cast())
        }
    }

    pub fn concat(other: &RawString) -> ObjectPtr {}
}

impl PartialEq for RawString {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let my_slice = std::slice::from_raw_parts_mut(self.chars, self.len);
            let other_slice = std::slice::from_raw_parts_mut(other.chars, other.len);
            my_slice == other_slice
        }
    }
}
