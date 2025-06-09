mod gc;
mod gc_roots;
mod gc_scoped_root;
mod markable;

pub use gc::{GC, Gc};
pub(crate) use gc_roots::GcRoots;
pub(crate) use markable::Markable;
