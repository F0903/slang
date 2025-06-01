mod dealloc;
mod drop_dealloc;
mod global_alloc;
mod heap_ptr;

use std::{
    alloc::{GlobalAlloc, Layout, System, handle_alloc_error},
    ptr::null_mut,
};

pub use dealloc::Dealloc;
pub use drop_dealloc::{DeallocOnDrop, DropDealloc};
pub use heap_ptr::HeapPtr;

fn allocate<T>(layout: Layout) -> *mut T {
    unsafe { System.alloc(layout).cast() }
}

fn free<T>(ptr: *mut T, layout: Layout) {
    unsafe {
        System.dealloc(ptr.cast(), layout);
    }
}

pub fn reallocate<T>(mut ptr: *mut u8, old_cap: usize, new_cap: usize) -> *mut u8 {
    let old_layout = Layout::array::<T>(old_cap).unwrap();
    let new_layout = Layout::array::<T>(new_cap).unwrap();

    if new_cap == 0 {
        free(ptr, old_layout);
        return null_mut();
    }

    if ptr.is_null() {
        ptr = allocate(new_layout);
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
