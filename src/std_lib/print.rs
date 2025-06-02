use crate::native_functions;

pub fn setup() -> &'static [NativeFunction] {
    FUNCTIONS
}

native_functions! {
    #[arity(1)]
    pub fn print_line(args: &[Value]) -> Result<Value> {
        let val = &args[0];
        println!("{}", val);
        Ok(Value::None)
    }
}
