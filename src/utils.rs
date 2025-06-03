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

#[macro_export]
macro_rules! unwrap_enum {
    // With custom error message
    ($var:expr, $variant:path, $msg:expr) => {
        match $var {
            $variant(val) => val,
            _ => unreachable!("{}", $msg),
        }
    };
    // Without custom error message
    ($var:expr, $variant:path) => {
        match $var {
            $variant(val) => val,
            _ => unreachable!("Expected {} variant in unwrap_enum", stringify!($variant)),
        }
    };
}
