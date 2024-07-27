#[macro_export]
macro_rules! test_debug {
    ($($arg:tt)*) => {{
        if std::env::var("RUST_TEST").is_ok() {
            println!($($arg)*);
        }
    }};
}
