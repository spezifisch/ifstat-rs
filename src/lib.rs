#[macro_use]
extern crate lazy_static;

#[macro_use]
mod utils;

use clap::Parser;
use std::collections::HashMap;
use std::env;
use std::io;
#[cfg(target_os = "macos")]
use std::process::Command;
#[cfg(target_os = "windows")]
use std::slice;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, FALSE, NO_ERROR};
#[cfg(target_os = "windows")]
use windows::Win32::NetworkManagement::IpHelper::{GetIfTable, MIB_IFTABLE};

#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::io::BufReader;

const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");

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

    /// Hides interfaces with zero counters.
    #[clap(short = 'z')]
    pub hide_zero_counters: bool,

    /// Delay between updates in seconds (default is 1 second)
    #[clap(default_value = "1")]
    pub delay: f64,

    /// Delay before the first measurement in seconds (default is same as --delay)
    #[clap(long)]
    pub first_measurement: Option<f64>,

    /// Number of updates before stopping (default is unlimited)
    pub count: Option<u64>,
}

lazy_static! {
    static ref LONG_VERSION: String = {
        // get build config
        let commit_hash = option_env!("VERGEN_GIT_SHA").unwrap_or("unknown");
        let git_dirty = option_env!("VERGEN_GIT_DIRTY").unwrap_or("unknown");
        let build_timestamp = option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("unknown");
        let rust_version = option_env!("VERGEN_RUSTC_SEMVER").unwrap_or("unknown");
        let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

        // build git commit string
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

fn filter_zero_counters(stats: &HashMap<String, (u64, u64)>, interfaces: &[String]) -> Vec<String> {
    interfaces
        .iter()
        .filter(|iface| {
            if let Some(&(rx, tx)) = stats.get(*iface) {
                rx != 0 || tx != 0
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

#[cfg(target_os = "linux")]
pub fn get_net_dev_stats() -> Result<HashMap<String, (u64, u64)>, std::io::Error> {
    let file = File::open("/proc/net/dev")?;
    let reader = BufReader::new(file);
    parse_net_dev_stats(reader)
}

#[cfg(target_os = "macos")]
pub fn get_net_dev_stats() -> Result<HashMap<String, (u64, u64)>, std::io::Error> {
    let output = Command::new("netstat")
        .arg("-b")
        .output()
        .expect("Failed to execute netstat command");
    let reader = BufReader::new(output.stdout.as_slice());
    parse_net_dev_stats(reader)
}

#[cfg(target_os = "windows")]
pub fn get_net_dev_stats() -> std::result::Result<HashMap<String, (u64, u64)>, std::io::Error> {
    let mut size = 0;

    unsafe {
        // First call to GetIfTable to get the necessary buffer size
        let result = GetIfTable(None, &mut size, FALSE);
        if result != ERROR_INSUFFICIENT_BUFFER.0 {
            eprintln!("Initial GetIfTable call failed: {}", result);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to get buffer size for network interface table",
            ));
        }
    }

    // Allocate the necessary buffer
    let mut buffer: Vec<u8> = vec![0; size as usize];
    let table: *mut MIB_IFTABLE = buffer.as_mut_ptr() as *mut MIB_IFTABLE;

    unsafe {
        let result = GetIfTable(Some(table), &mut size, FALSE);
        if result != NO_ERROR.0 {
            eprintln!("GetIfTable call failed: {}", result);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to get network interface table",
            ));
        }

        let table_ref = &*table;
        let mut stats = HashMap::new();

        // Create a slice from the dynamically sized array
        let rows = slice::from_raw_parts(table_ref.table.as_ptr(), table_ref.dwNumEntries as usize);

        for row in rows {
            let iface_name_utf16: Vec<u16> = row
                .wszName
                .iter()
                .take_while(|&&c| c != 0) // Stop at the first null character
                .cloned()
                .collect();
            let iface_name = String::from_utf16_lossy(&iface_name_utf16)
                .trim()
                .to_string();
            let rx_bytes = row.dwInOctets as u64;
            let tx_bytes = row.dwOutOctets as u64;

            test_debug!(
                "Interface: {}, RX: {}, TX: {}",
                iface_name,
                rx_bytes,
                tx_bytes
            );

            stats.insert(iface_name, (rx_bytes, tx_bytes));
        }

        Ok(stats)
    }
}

pub fn parse_net_dev_stats<R: io::BufRead>(
    reader: R,
) -> Result<HashMap<String, (u64, u64)>, std::io::Error> {
    let mut stats = HashMap::new();
    let lines: Vec<_> = reader.lines().collect::<Result<_, _>>()?;
    test_debug!("Parsing {} lines", lines.len());

    for (_index, line) in lines.into_iter().enumerate().skip(2) {
        test_debug!("Parsing line: {}", line);
        if let Some((iface, rest)) = line.split_once(':') {
            let fields: Vec<&str> = rest.split_whitespace().collect();
            if fields.len() >= 9 {
                let rx_bytes: u64 = fields[0].parse().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid RX bytes")
                })?;
                let tx_bytes: u64 = fields[8].parse().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid TX bytes")
                })?;
                stats.insert(iface.trim().to_string(), (rx_bytes, tx_bytes));
            } else {
                test_debug!(
                    "Invalid line format: '{}' ({} fields: {:?})",
                    line,
                    fields.len(),
                    fields
                );
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid line format: {} fields", fields.len()),
                ));
            }
        } else {
            test_debug!("Invalid line format: '{}' (no colon found)", line);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid line format (no colon found)",
            ));
        }
    }
    Ok(stats)
}

fn shorten_name(name: &str) -> String {
    if name.len() > 16 {
        let start = name.find('\\').map(|i| i + 1).unwrap_or(0);
        let name = &name[start..];

        // assume form like \DEVICE\TCPIP_{2EE2C70C-A092-4D88-A654-98C8D7645CD5}
        if let Some(start_idx) = name.find("TCPIP_{") {
            let prefix_len = start_idx + 1 + 7 + 4; // length of "TCPIP_{0737"
            if prefix_len < name.len() {
                let suffix_start = name.len().saturating_sub(5);
                let prefix = &name[start_idx..prefix_len];
                let suffix = &name[suffix_start..];

                return format!("{}..{}", prefix, suffix);
            }
        }

        // If the name doesn't match the expected pattern or prefix_len check fails
        if name.len() > 13 {
            return format!("{}...", &name[..13]);
        }
    }
    // If the name length is 16 or less, or all other conditions fail
    name.to_string()
}

pub fn print_headers(
    interfaces: &[String],
    writer: &mut dyn std::io::Write,
    hide_zero_counters: bool,
    stats: &HashMap<String, (u64, u64)>,
) -> std::io::Result<()> {
    let interfaces = if hide_zero_counters {
        filter_zero_counters(stats, interfaces)
    } else {
        interfaces.to_vec()
    };

    if interfaces.is_empty() {
        return Ok(());
    }

    let width = 18; // width for each interface field including space for in/out
    for (i, interface) in interfaces.iter().enumerate() {
        let short_interface = shorten_name(interface);
        let padded_name = format!("{:^width$}", short_interface, width = width);
        write!(writer, "{}", padded_name)?;
        if i < interfaces.len() - 1 {
            write!(writer, "  ")?; // additional spaces between columns
        }
    }
    writeln!(writer)?;

    for (i, _) in interfaces.iter().enumerate() {
        write!(writer, "{:>8}  {:>8}", "KB/s in", "KB/s out")?;
        if i < interfaces.len() - 1 {
            write!(writer, "  ")?; // additional spaces between columns
        }
    }
    writeln!(writer)?;

    Ok(())
}

pub fn print_stats(
    previous: &HashMap<String, (u64, u64)>,
    current: &HashMap<String, (u64, u64)>,
    interfaces: &[String],
    writer: &mut dyn std::io::Write,
    hide_zero_counters: bool,
) -> std::io::Result<()> {
    let interfaces = if hide_zero_counters {
        filter_zero_counters(current, interfaces)
    } else {
        interfaces.to_vec()
    };

    for (i, interface) in interfaces.iter().enumerate() {
        if let (Some(&(prev_rx, prev_tx)), Some(&(cur_rx, cur_tx))) =
            (previous.get(interface), current.get(interface))
        {
            let rx_kbps = (cur_rx.saturating_sub(prev_rx)) as f64 / 1024.0;
            let tx_kbps = (cur_tx.saturating_sub(prev_tx)) as f64 / 1024.0;
            write!(writer, "{:>8.2}  {:>8.2}", rx_kbps, tx_kbps)?;
            if i < interfaces.len() - 1 {
                write!(writer, "  ")?; // additional spaces between columns
            }
        }
    }
    writeln!(writer)?;

    Ok(())
}
