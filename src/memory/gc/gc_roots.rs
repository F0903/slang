use std::fmt::Debug;

use crate::memory::Gc;

/// A trait for registering GC roots.
pub(crate) trait GcRoots {
    fn mark_roots(&mut self, gc: &Gc);
}

impl Debug for (dyn GcRoots + 'static) {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GcRoots")
    }
}
