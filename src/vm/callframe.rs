use crate::{
    collections::StackOffset,
    value::{Value, object::Function},
};

pub struct CallFrame {
    pub function: Function,
    pub ip: *mut u8,
    pub stack_offset: usize,
}
