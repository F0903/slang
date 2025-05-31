use crate::value::object::Function;

pub struct CallFrame {
    pub function: Function,
    pub ip: *mut u8,
    pub stack_base_offset: usize,
}
