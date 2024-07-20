use std::fmt::Debug;

use super::object::Object;

pub(super) union ValueCasts {
    pub(super) boolean: bool,
    pub(super) number: f64,
    pub(super) object: *mut Object,
}

impl Debug for ValueCasts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.write_fmt(format_args!(
                "ValueCasts: bool=[{}] f64=[{:.5}]",
                self.boolean, self.number
            ))
        }
    }
}

impl Clone for ValueCasts {
    fn clone(&self) -> Self {
        // Might be wise to construct this with the largest field
        unsafe {
            ValueCasts {
                number: self.number,
            }
        }
    }
}
