use crate::vm::Vm;

mod print;
mod time;

#[macro_export]
macro_rules! native_functions {
    (
        $(
            #[arity($arity:literal)]
            pub fn $name:ident($args:ident : &[Value]) -> Result<Value> $body:block
        )*
    ) => {
        use crate::{error::Result, value::{Value, object::NativeFunction}};

        $(
            pub fn $name($args: &[Value]) -> Result<Value> $body
        )*

        static FUNCTIONS: &[NativeFunction] = &[
            $(
                NativeFunction::new($name, $arity, stringify!($name)),
            )*
        ];
    }
}

pub fn init(vm: &mut Vm) {
    let print_funcs = print::setup();
    vm.register_native_functions(print_funcs);

    let time_funcs = time::setup();
    vm.register_native_functions(time_funcs);
}
