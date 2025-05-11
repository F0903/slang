use std::alloc::{GlobalAlloc, Layout, System};

#[global_allocator]
static A: AllocWrapper = AllocWrapper;

struct AllocWrapper;
unsafe impl GlobalAlloc for AllocWrapper {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}
