use super::ObjectNode;
use crate::memory::HeapPtr;

pub(super) union ValueCasts {
    pub(super) boolean: bool,
    pub(super) number: f64,
    pub(super) object_node: HeapPtr<ObjectNode>,
}

impl Clone for ValueCasts {
    fn clone(&self) -> Self {
        // IMPORTANT: initialize with the largest value
        unsafe {
            ValueCasts {
                object_node: self.object_node,
            }
        }
    }
}
