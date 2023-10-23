use std::{
    alloc::{handle_alloc_error, Layout},
    ptr::null_mut,
};

pub fn reallocate<T>(mut ptr: *mut u8, old_cap: usize, new_cap: usize) -> *mut u8 {
    let old_layout = Layout::array::<T>(old_cap).unwrap();
    let new_layout = Layout::array::<T>(new_cap).unwrap();

    if new_cap == 0 {
        unsafe {
            std::alloc::dealloc(ptr, old_layout);
        }
        return null_mut();
    }

    if ptr.is_null() {
        ptr = unsafe { std::alloc::alloc(new_layout) };
        if ptr.is_null() {
            handle_alloc_error(old_layout);
        }
        return ptr;
    }

    let new_block = unsafe { std::alloc::realloc(ptr, old_layout, new_layout.size()) };
    if new_block.is_null() {
        handle_alloc_error(old_layout);
    }
    new_block
}
