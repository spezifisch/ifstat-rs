use clap::Parser;
use std::env;

const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");

lazy_static::lazy_static! {
    static ref LONG_VERSION: String = {
        // Get build config
        let commit_hash = option_env!("VERGEN_GIT_SHA").unwrap_or("unknown");
        let git_dirty = option_env!("VERGEN_GIT_DIRTY").unwrap_or("unknown");
        let build_timestamp = option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("unknown");
        let rust_version = option_env!("VERGEN_RUSTC_SEMVER").unwrap_or("unknown");
        let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

        // Build git commit string
        let commit_str = if commit_hash.starts_with("VERGEN") {
            "non-git build".to_string()
        } else {
            let suffix = if git_dirty == "false" { "" } else { "-dirty" };
            format!("{}{}", commit_hash, suffix)
        };

        format!(
            "A tool to report network interface statistics.\n\n\
            Author: {}\n\
            Repo: {}\n\
            License: {}\n\
            Commit: {}\n\
            Build Timestamp: {}\n\
            Rust Version: {}\n\
            Compilation Target: {}",
            AUTHOR, REPO_URL, LICENSE, commit_str, build_timestamp, rust_version, target
        )
    };
}

#[derive(Parser)]
#[clap(version = VERSION, author = AUTHOR, long_version = LONG_VERSION.as_str())]
pub struct Opts {
    /// Interfaces to monitor, separated by commas (e.g., "eth0,lo")
    #[clap(short, long)]
    pub interfaces: Option<String>,

    /// Enables monitoring of all interfaces found for which statistics are available.
    #[clap(short = 'a')]
    pub monitor_all: bool,

    /// Enables monitoring of loopback interfaces for which statistics are available.
    #[clap(short = 'l')]
    pub monitor_loopback: bool,

    /// Hides interfaces with zero counters (default false on Linux/Mac, true in Windows).
    #[clap(
        short = 'z',
        default_value_if("cfg(target_os = \"windows\")", "false", "true")
    )]
    pub hide_zero_counters: bool,

    /// List all available network interfaces and exit
    #[clap(long)]
    pub list_interfaces: bool,

    /// Delay between updates in seconds (default is 1 second)
    #[clap(default_value = "1")]
    pub delay: f64,

    /// Delay before the first measurement in seconds (default is same as --delay)
    #[clap(long)]
    pub first_measurement: Option<f64>,

    /// Number of updates before stopping (default is unlimited)
    pub count: Option<u64>,
}
