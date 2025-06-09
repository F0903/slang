mod alloc;
mod dealloc;
mod drop_dealloc;
mod gc;
mod heap_ptr;

pub(crate) use alloc::reallocate;

pub use dealloc::Dealloc;
pub use drop_dealloc::{DeallocOnDrop, DropDealloc};
pub use gc::{GC, Gc, GcRoots};
pub use heap_ptr::HeapPtr;
