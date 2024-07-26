mod dealloc;
mod manual_ptr;

pub use dealloc::Dealloc;
pub use manual_ptr::ManualPtr;

use std::{
    alloc::{handle_alloc_error, GlobalAlloc, Layout, System},
    ptr::null_mut,
};

struct AllocWrapper;
unsafe impl GlobalAlloc for AllocWrapper {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static A: AllocWrapper = AllocWrapper;

/// Use ManualPtr instead where possible.
pub fn allocate<T>() -> *mut T {
    unsafe { System.alloc(Layout::new::<T>()).cast() }
}

pub fn free<T>(ptr: *mut T) {
    unsafe {
        System.dealloc(ptr.cast(), Layout::new::<T>());
    }
}

pub fn reallocate<T>(mut ptr: *mut u8, old_cap: usize, new_cap: usize) -> *mut u8 {
    let old_layout = Layout::array::<T>(old_cap).unwrap();
    let new_layout = Layout::array::<T>(new_cap).unwrap();

    if new_cap == 0 {
        unsafe {
            System.dealloc(ptr, old_layout);
        }
        return null_mut();
    }

    if ptr.is_null() {
        ptr = unsafe { System.alloc(new_layout) };
        if ptr.is_null() {
            handle_alloc_error(old_layout);
        }
        return ptr;
    }

    let new_block = unsafe { System.realloc(ptr, old_layout, new_layout.size()) };
    if new_block.is_null() {
        handle_alloc_error(old_layout);
    }
    new_block
}
