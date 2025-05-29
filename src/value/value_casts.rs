use std::{fmt::Debug, mem::MaybeUninit};

use super::ObjectNode;

pub(super) union ValueCasts {
    pub(super) boolean: bool,
    pub(super) number: f64,
    pub(super) object: MaybeUninit<ObjectNode>,
}

impl Debug for ValueCasts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.write_fmt(format_args!(
                "ValueCasts: bool=[{}] f64=[{:.5}] object=[{:?}]",
                self.boolean, self.number, self.object
            ))
        }
    }
}

impl Clone for ValueCasts {
    fn clone(&self) -> Self {
        // IMPORTANT: initialize with the largest value
        unsafe {
            ValueCasts {
                object: self.object,
            }
        }
    }
}
