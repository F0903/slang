use super::{raw_string::RawString, Object, ObjectType};
use crate::memory::reallocate;
use std::ffi::CStr;

pub struct ObjectPtr {
    obj: *mut Object,
}

// Prepare your body for wild unsafetiness
impl ObjectPtr {
    pub fn as_str(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.obj.cast()) }
    }

    pub fn get_type(&self) -> ObjectType {
        unsafe { self.obj.read().obj_type }
    }

    pub fn dealloc(&self) {
        unsafe {
            match self.obj.read().obj_type {
                ObjectType::RawString => {
                    let str = self.as_str();
                    reallocate::<RawString>(self.obj.cast(), str.count_bytes(), 0);
                }
            };
        }
    }
}

impl Drop for ObjectPtr {
    fn drop(&mut self) {
        self.dealloc();
    }
}

impl PartialEq for ObjectPtr {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.obj.read().obj_type == other.obj.read().obj_type && self.obj == other.obj }
    }
}

impl PartialOrd for ObjectPtr {
    fn gt(&self, other: &Self) -> bool {
        unsafe { self.obj.read().obj_type == other.obj.read().obj_type && self.obj > other.obj }
    }

    fn ge(&self, other: &Self) -> bool {
        unsafe { self.obj.read().obj_type == other.obj.read().obj_type && self.obj >= other.obj }
    }

    fn lt(&self, other: &Self) -> bool {
        unsafe { self.obj.read().obj_type == other.obj.read().obj_type && self.obj < other.obj }
    }

    fn le(&self, other: &Self) -> bool {
        unsafe { self.obj.read().obj_type == other.obj.read().obj_type && self.obj <= other.obj }
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self > other {
            Some(std::cmp::Ordering::Greater)
        } else if self < other {
            Some(std::cmp::Ordering::Less)
        } else {
            Some(std::cmp::Ordering::Equal)
        }
    }
}
