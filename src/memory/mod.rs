mod alloc;
mod dealloc;
mod drop_dealloc;
mod gc;
mod heap_ptr;
mod weak_ref;

pub(crate) use alloc::reallocate;

pub use dealloc::Dealloc;
pub use drop_dealloc::{DeallocOnDrop, DropDealloc};
pub use gc::{GC, Gc};
pub(crate) use gc::{GcRoots, Markable};
pub use heap_ptr::HeapPtr;
pub use weak_ref::WeakRef;
