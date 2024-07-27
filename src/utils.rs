#[cfg(test)]
#[macro_export]
macro_rules! test_debug {
    ($($arg:tt)*) => (println!($($arg)*));
}

#[cfg(not(test))]
#[macro_export]
macro_rules! test_debug {
    ($($arg:tt)*) => {};
}
