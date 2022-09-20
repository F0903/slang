#[macro_export]
macro_rules! create_string_map {
    ($($key:literal => $val:expr),+) => {{
        use ::std::collections::HashMap;
        let mut map = HashMap::new();
        $(
            map.insert($key.to_owned(), $val);
        )*
        map
    }};
}
