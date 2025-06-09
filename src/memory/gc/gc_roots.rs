use crate::memory::Gc;

/// A trait for registering GC roots.
pub(crate) trait GcRoots {
    fn mark_roots(&mut self, gc: &Gc);
}
