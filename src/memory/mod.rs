mod gc;

pub use gc::{DropDealloc, GC, Gc, GcPtr};
pub(crate) use gc::{MarkRoots, RootMarker};
