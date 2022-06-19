use crate::vm::VirtualMachine;

mod print;

pub fn register_funcs(vm: &mut VirtualMachine) {
    print::register_funcs(vm);
}
