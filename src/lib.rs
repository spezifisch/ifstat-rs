use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");

#[derive(Parser)]
#[clap(version = "1.0", author = AUTHOR, long_version = LONG_VERSION.as_str())]
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
        let commit_hash = option_env!("VERGEN_GIT_SHA").unwrap_or("unknown");
        let build_timestamp = option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("unknown");
        let rust_version = option_env!("VERGEN_RUSTC_SEMVER").unwrap_or("unknown");
        let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

        format!(
            "A tool to report network interface statistics.\n\n\
            Author: {}\n\
            License: {}\n\
            Build info:\n\
            Commit: {}\n\
            Build Timestamp: {}\n\
            Rust Version: {}\n\
            Compilation Target: {}\n\
            Repo: {}",
            AUTHOR, LICENSE, commit_hash, build_timestamp, rust_version, target, REPO_URL
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

pub fn get_net_dev_stats<R: BufRead>(
    reader: R,
) -> Result<HashMap<String, (u64, u64)>, std::io::Error> {
    let mut stats = HashMap::new();
    let re = Regex::new(r"^\s*([^:]+):\s*(\d+)\s+(?:\d+\s+){7}(\d+)\s+").unwrap();

    for line in reader.lines().skip(2) {
        let line = line?;
        if let Some(caps) = re.captures(&line) {
            let interface = caps[1].to_string();
            let rx_bytes: u64 = caps[2].parse().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid RX bytes")
            })?;
            let tx_bytes: u64 = caps[3].parse().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid TX bytes")
            })?;
            stats.insert(interface, (rx_bytes, tx_bytes));
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid line format",
            ));
        }
    }
    Ok(stats)
}

#[cfg(target_os = "linux")]
pub fn get_net_dev_stats_from_file() -> Result<HashMap<String, (u64, u64)>, std::io::Error> {
    let file = File::open("/proc/net/dev")?;
    let reader = BufReader::new(file);
    get_net_dev_stats(reader)
}

#[cfg(target_os = "macos")]
pub fn get_net_dev_stats_from_file() -> Result<HashMap<String, (u64, u64)>, std::io::Error> {
    use std::process::Command;

    let output = Command::new("netstat").arg("-bI").arg("en0").output()?;

    let data = String::from_utf8_lossy(&output.stdout);
    let mut stats = HashMap::new();
    let re = Regex::new(r"^\s*([^:]+):\s*(\d+)\s+(?:\d+\s+){6}\d+\s+(\d+)\s+").unwrap();

    for line in data.lines().skip(1) {
        if let Some(caps) = re.captures(line) {
            let interface = caps[1].to_string();
            let rx_bytes: u64 = caps[2].parse().unwrap_or(0);
            let tx_bytes: u64 = caps[3].parse().unwrap_or(0);
            stats.insert(interface, (rx_bytes, tx_bytes));
        }
    }
    Ok(stats)
}

#[cfg(target_os = "windows")]
pub fn get_net_dev_stats_from_file() -> Result<HashMap<String, (u64, u64)>, std::io::Error> {
    use std::ptr::null_mut;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::iphlpapi::GetIfTable;
    use winapi::um::iphlpapi::MIB_IFROW;
    use winapi::um::iphlpapi::MIB_IFTABLE;

    let mut size: DWORD = 0;
    unsafe {
        GetIfTable(null_mut(), &mut size, 0);
    }
    let mut table: Vec<u8> = vec![0; size as usize];
    let table_ptr = table.as_mut_ptr() as *mut MIB_IFTABLE;

    let result = unsafe { GetIfTable(table_ptr, &mut size, 0) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }

    let table = unsafe { &*table_ptr };
    let mut stats = HashMap::new();
    for i in 0..table.dwNumEntries {
        let row = unsafe { &*table.table.as_ptr().add(i as usize) };
        let interface = format!("{}", row.dwIndex);
        let rx_bytes: u64 = row.dwInOctets as u64;
        let tx_bytes: u64 = row.dwOutOctets as u64;
        stats.insert(interface, (rx_bytes, tx_bytes));
    }
    Ok(stats)
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
        let padded_name = format!("{:^width$}", interface, width = width);
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
