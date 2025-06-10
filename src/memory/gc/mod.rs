mod drop_dealloc;
mod gc;
mod gc_ptr;
mod root_marker;
mod scoped_root_object;

pub use drop_dealloc::DropDealloc;
pub use gc::{GC, Gc};
pub use gc_ptr::GcPtr;
pub(crate) use root_marker::{MarkRoots, RootMarker};
pub use scoped_root_object::ScopedRootObject;
