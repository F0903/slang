use {
    super::Object,
    crate::{memory::reallocate, vm::GLOBAL_VM},
    std::ptr::null_mut,
};

#[repr(C)]
pub struct RawString {
    parent: Object,
    len: usize,
    chars: *mut u8, // Heap alloced, will need to be freed.
}

impl RawString {
    pub const fn get_char_ptr(&self) -> *mut u8 {
        self.chars
    }

    pub const fn get_len(&self) -> usize {
        self.len
    }

    fn alloc_string() -> (*mut Self, Self) {
        unsafe {
            let objects_head = GLOBAL_VM.get_objects_head();
            let ptr = reallocate::<RawString>(null_mut(), 0, 1);
            let obj_ptr = ptr.cast::<RawString>();
            let mut obj = obj_ptr.read();
            obj.parent = Object {
                obj_type: super::ObjectType::String,
                next: objects_head,
            };
            GLOBAL_VM.set_objects_head(obj_ptr.cast());
            (obj_ptr, obj)
        }
    }

    fn from_take_mem(mem: *mut u8, len: usize) -> *mut RawString {
        let (obj_ptr, mut obj) = Self::alloc_string();
        obj.chars = mem;
        obj.len = len;
        obj_ptr
    }

    pub fn alloc_from_str(value: &str) -> *mut RawString {
        unsafe {
            let (obj_ptr, mut obj) = Self::alloc_string();
            obj.chars = reallocate::<u8>(obj.chars, 0, value.len());
            value
                .as_ptr()
                .copy_to_nonoverlapping(obj.chars, value.len());
            obj.len = value.len();
            obj_ptr
        }
    }

    pub fn concat(&self, other: &RawString) -> *mut RawString {
        // Consider using Vec as in normal rust after GC has been implemented in global alloc.
        let buf_len = self.len + other.len;
        let buf = reallocate::<u8>(null_mut(), 0, buf_len);
        unsafe {
            self.chars.copy_to_nonoverlapping(buf, self.len);
            other.chars.copy_to_nonoverlapping(buf, buf_len);
        }
        Self::from_take_mem(buf, buf_len)
    }
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
