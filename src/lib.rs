use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use lazy_static::lazy_static;

const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

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

        format!(
            "ifstat-rs: A tool to report network interface statistics.\n\n\
            Built with Rust.\n\n\
            Build info:\n\
            Commit: {}\n\
            Build Timestamp: {}\n\
            Rust Version: {}\n\
            Repo: {}",
            commit_hash, build_timestamp, rust_version, REPO_URL
        )
    };
}

pub fn get_net_dev_stats() -> Result<HashMap<String, (u64, u64)>, std::io::Error> {
    let file = File::open("/proc/net/dev")?;
    let reader = BufReader::new(file);
    let mut stats = HashMap::new();
    let re = Regex::new(r"^\s*([^:]+):\s*(\d+)\s+.*\s+(\d+)\s+").unwrap();

    for line in reader.lines().skip(2) {
        let line = line?;
        if let Some(caps) = re.captures(&line) {
            let interface = caps[1].to_string();
            let rx_bytes: u64 = caps[2].parse().unwrap_or(0);
            let tx_bytes: u64 = caps[3].parse().unwrap_or(0);
            stats.insert(interface, (rx_bytes, tx_bytes));
        }
    }
    Ok(stats)
}

pub fn print_headers(interfaces: &[String], writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    if interfaces.is_empty() {
        return Ok(());
    }

    write!(writer, "{:>10} ", " ")?;
    for interface in interfaces {
        write!(writer, "{:<10} {:<10}  ", interface, " ")?;
    }
    writeln!(writer)?;

    write!(writer, "{:>10} ", " ")?;
    for _ in interfaces {
        write!(writer, "{:<10} {:<10}  ", "KB/s in", "KB/s out")?;
    }
    writeln!(writer)?;

    Ok(())
}

pub fn print_stats(
    previous: &HashMap<String, (u64, u64)>,
    current: &HashMap<String, (u64, u64)>,
    interfaces: &[String],
    writer: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    let interface_names: Vec<_> = interfaces.to_vec();

    if interface_names.is_empty() {
        return Ok(());
    }

    write!(writer, "{:>10} ", " ")?;

    for interface in &interface_names {
        if let (Some(&(prev_rx, prev_tx)), Some(&(cur_rx, cur_tx))) =
            (previous.get(interface), current.get(interface))
        {
            let rx_kbps = (cur_rx.saturating_sub(prev_rx)) as f64 / 1024.0;
            let tx_kbps = (cur_tx.saturating_sub(prev_tx)) as f64 / 1024.0;
            write!(writer, "{:<10.2} {:<10.2}  ", rx_kbps, tx_kbps)?;
        }
    }
    writeln!(writer)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufWriter;

    fn mock_net_dev_data() -> String {
        "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 123456789 98765    0    0    0     0          0         0 123456789 98765    0    0    0     0       0          0
  eth0: 987654321 56789    0    0    0     0          0         0 987654321 56789    0    0    0     0       0          0
".to_string()
    }

    // Mock get_net_dev_stats function
    fn get_mock_net_dev_stats() -> Result<HashMap<String, (u64, u64)>, std::io::Error> {
        let data = mock_net_dev_data();
        let reader = BufReader::new(data.as_bytes());
        let mut stats = HashMap::new();
        let re = Regex::new(r"^\s*([^:]+):\s*(\d+)\s+.*\s+(\d+)\s+").unwrap();

        for line in reader.lines().skip(2) {
            let line = line?;
            if let Some(caps) = re.captures(&line) {
                let interface = caps[1].to_string();
                let rx_bytes: u64 = caps[2].parse().unwrap_or(0);
                let tx_bytes: u64 = caps[3].parse().unwrap_or(0);
                stats.insert(interface, (rx_bytes, tx_bytes));
            }
        }
        Ok(stats)
    }

    #[test]
    fn test_parse_net_dev_stats() {
        let stats = get_mock_net_dev_stats().unwrap();
        assert_eq!(stats["lo"], (123456789, 123456789));
        assert_eq!(stats["eth0"], (987654321, 987654321));
    }

    #[test]
    fn test_print_headers() {
        let interfaces = vec!["eth0".to_string(), "lo".to_string()];
        let expected = "\
 eth0                 lo
 KB/s in  KB/s out   KB/s in  KB/s out
";
        let mut output = Vec::new();
        {
            let mut writer = BufWriter::new(&mut output);
            print_headers(&interfaces, &mut writer).unwrap();
        }
        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }

    #[test]
    fn test_print_stats() {
        let previous_stats = get_mock_net_dev_stats().unwrap();
        let current_stats = get_mock_net_dev_stats().unwrap();
        let interfaces = vec!["eth0".to_string(), "lo".to_string()];
        let expected = "\
    0.00       0.00        0.00       0.00
";
        let mut output = Vec::new();
        {
            let mut writer = BufWriter::new(&mut output);
            print_stats(&previous_stats, &current_stats, &interfaces, &mut writer).unwrap();
        }
        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }

    #[test]
    fn test_command_line_options() {
        let opts = Opts::try_parse_from(&[
            "test",
            "-i",
            "eth0,lo",
            "--first-measurement",
            "0.5",
            "--delay",
            "1.0",
            "--count",
            "10",
        ]).unwrap();
        assert_eq!(opts.interfaces.unwrap(), "eth0,lo");
        assert_eq!(opts.first_measurement.unwrap(), 0.5);
        assert_eq!(opts.delay, 1.0);
        assert_eq!(opts.count.unwrap(), 10);
    }
}