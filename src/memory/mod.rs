mod alloc;
mod dealloc;
mod drop_dealloc;
mod global_alloc;
mod heap_ptr;

pub use alloc::reallocate;

pub use dealloc::Dealloc;
pub use drop_dealloc::{DeallocOnDrop, DropDealloc};
pub use heap_ptr::HeapPtr;
