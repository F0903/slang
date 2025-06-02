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
    ($var:ident, $variant:path) => {
        match $var {
            $variant(val) => val,
            _ => unreachable!("Expected {} variant", stringify!($variant)),
        }
    };
}
