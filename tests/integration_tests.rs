use std::collections::HashMap;
use std::io::{self, BufRead, BufReader};

use regex::Regex;
use clap::Parser;
use ifstat_rs::{print_headers, print_stats, Opts};

fn mock_net_dev_data() -> String {
    "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 123456789 98765    0    0    0     0          0         0 123456789 98765    0    0    0     0       0          0
  eth0: 987654321 56789    0    0    0     0          0         0 987654321 56789    0    0    0     0       0          0
".to_string()
}

// Mock get_net_dev_stats function
fn get_mock_net_dev_stats() -> Result<HashMap<String, (u64, u64)>, io::Error> {
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
_______eth0________________lo_______
_KB/s_in__KB/s_out__KB/s_in__KB/s_out
";
    let mut output = Vec::new();
    {
        let mut writer = std::io::BufWriter::new(&mut output);
        print_headers(&interfaces, &mut writer).unwrap();
    }
    let output_str = String::from_utf8(output).unwrap().replace(' ', "_");
    assert_eq!(output_str, expected);
}

#[test]
fn test_print_stats() {
    let previous_stats = get_mock_net_dev_stats().unwrap();
    let current_stats = get_mock_net_dev_stats().unwrap();
    let interfaces = vec!["eth0".to_string(), "lo".to_string()];
    let expected = "\
____0.00______0.00______0.00______0.00
";
    let mut output = Vec::new();
    {
        let mut writer = std::io::BufWriter::new(&mut output);
        print_stats(&previous_stats, &current_stats, &interfaces, &mut writer).unwrap();
    }
    let output_str = String::from_utf8(output).unwrap().replace(' ', "_");
    assert_eq!(output_str, expected);
}

#[test]
fn test_command_line_options() {
    let opts = Opts::try_parse_from(&[
        "test",
        "-i",
        "eth0,lo",
        "--first-measurement",
        "0.5",
        "--delay=1.0",
        "--count=10",
    ]).unwrap();
    assert_eq!(opts.interfaces.unwrap(), "eth0,lo");
    assert_eq!(opts.first_measurement.unwrap(), 0.5);
    assert_eq!(opts.delay, 1.0);
    assert_eq!(opts.count.unwrap(), 10);
}
