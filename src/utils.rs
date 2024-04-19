#[macro_export]
macro_rules! fprint {
    ($args:tt) => {{
        print!($args);
        _ = std::io::stdout().flush();
    }};
}
