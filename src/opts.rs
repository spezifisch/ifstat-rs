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

    /// Delay before the first measurement in seconds (must be >= 0 if set)
    #[arg(long, value_parser = parse_non_negative_f64)]
    pub first_measurement: Option<f64>,

    /// Delay between updates in seconds (must be > 0)
    #[arg(default_value = "1", value_parser = parse_positive_f64)]
    pub delay: f64,

    /// Number of updates before stopping (must be > 0 if set)
    #[arg(value_parser = parse_positive_u64)]
    pub count: Option<u64>,
}

fn parse_positive_f64(src: &str) -> Result<f64, String> {
    let val: f64 = src
        .parse()
        .map_err(|_| format!("`{}` is not a valid number", src))?;
    if val <= 0.0 {
        Err(format!("`{}` must be greater than 0", src))
    } else {
        Ok(val)
    }
}

fn parse_non_negative_f64(src: &str) -> Result<f64, String> {
    let val: f64 = src
        .parse()
        .map_err(|_| format!("`{}` is not a valid number", src))?;
    if val < 0.0 {
        Err(format!("`{}` must be greater than or equal to 0", src))
    } else {
        Ok(val)
    }
}

fn parse_positive_u64(src: &str) -> Result<u64, String> {
    let val: u64 = src
        .parse()
        .map_err(|_| format!("`{}` is not a valid number > 0", src))?;
    if val == 0 {
        Err(format!("`{}` must be greater than 0", src))
    } else {
        Ok(val)
    }
}
