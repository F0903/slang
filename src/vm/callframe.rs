use std::fmt::Display;

use crate::value::object::Closure;

pub struct CallFrame {
    pub closure: Closure,
    pub ip: *mut u8,
    pub stack_base_offset: usize,
}

impl Display for CallFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Call Frame -> {:?} + {} | {}",
            self.ip, self.stack_base_offset, self.closure
        ))
    }
}
