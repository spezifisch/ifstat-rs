use vergen::{Config, vergen};

fn main() {
    vergen(Config::default()).expect("Unable to generate build information");
}
