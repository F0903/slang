use std::fmt::Debug;

use crate::memory::{Gc, GcPtr};

pub(crate) trait MarkRoots {
    fn mark_roots(&mut self, gc: &Gc);
}

pub(crate) struct RootMarker {
    mark_roots: GcPtr<dyn MarkRoots>,
}

impl RootMarker {
    pub const fn new(mark_roots: GcPtr<dyn MarkRoots>) -> Self {
        Self { mark_roots }
    }

    pub fn mark_roots(&mut self, gc: &Gc) {
        self.mark_roots.mark_roots(gc);
    }

    pub(super) fn get_marker_address(&self) -> usize {
        self.mark_roots.get_address()
    }
}

impl PartialEq for RootMarker {
    fn eq(&self, other: &Self) -> bool {
        self.mark_roots == other.mark_roots
    }
}

impl Debug for RootMarker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("RootMarker {:?}", self.mark_roots.get_raw()))
    }
}
