use crate::interpreter::Interpreter;

mod io_utils;

pub fn register(interpreter: &mut Interpreter) {
    io_utils::register(interpreter)
}
