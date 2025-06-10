mod drop_dealloc;
mod gc;
mod gc_ptr;
mod gc_roots;
mod gc_scoped_root;
mod markable;

pub use drop_dealloc::DropDealloc;
pub use gc::{GC, Gc};
pub use gc_ptr::GcPtr;
pub(crate) use gc_roots::GcRoots;
pub(crate) use markable::Markable;
