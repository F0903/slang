#[macro_export]
macro_rules! fprint {
    ($args:tt) => {{
        print!($args);
        std::io::stdout().flush();
    }};
}
