use std::path::Path;
use vergen::{vergen, Config};

fn main() {
    if Path::new(".git").exists() {
        vergen(Config::default()).expect("Unable to generate build information");
    } else {
        println!(
            "cargo:warning=No .git directory found. Skipping vergen build information generation."
        );
    }
}
