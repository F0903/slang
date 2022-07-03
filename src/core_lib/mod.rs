use crate::vm::VirtualMachine;

mod print;

pub fn register_funcs(vm: &mut VirtualMachine) {
    print::register_funcs(vm);
    vm.register_native_func("test_ret", test_ret);
}

pub fn test_ret(_input: Vec<crate::types::Argument>) -> crate::types::Value {
    crate::types::Value::String("HELLO FROM RUST!!!".into())
}
