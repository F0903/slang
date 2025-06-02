use std::{sync::LazyLock, time::Instant};

use crate::native_functions;

static START_TIME: LazyLock<Instant> = std::sync::LazyLock::new(Instant::now);

pub fn setup() -> &'static [NativeFunction] {
    LazyLock::force(&START_TIME);
    FUNCTIONS
}

native_functions! {
    #[arity(0)]
    pub fn time_since_start(_args: &[Value]) -> Result<Value> {
        let elapsed = START_TIME.elapsed().as_secs_f64();
        Ok(Value::Number(elapsed))
    }
}
