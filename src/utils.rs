#[macro_export]
macro_rules! fprint {
    ($args:tt) => {{
        print!($args);
        _ = std::io::stdout().flush();
    }};
}

#[macro_export]
macro_rules! dbg_println {
    ($($arg:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            println!($($arg)*);
        }
    }};
}
