mod alloc;
mod drop_dealloc;
mod gc;
mod heap_ptr;

pub(crate) use alloc::reallocate;

pub use drop_dealloc::DropDealloc;
pub use gc::{GC, Gc};
pub(crate) use gc::{GcRoots, Markable};
pub use heap_ptr::HeapPtr;
