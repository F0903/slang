mod gc;
mod heap_ptr;

pub use gc::{DropDealloc, GC, Gc, GcPtr};
pub(crate) use gc::{GcScopedRoot, MarkRoots, RootMarker};
pub(crate) use heap_ptr::HeapPtr;
