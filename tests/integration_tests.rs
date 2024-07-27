use std::collections::HashMap;
use std::io::{self, BufRead, BufReader};
use clap::Parser;
use ifstat_rs::{print_headers, print_stats, get_net_dev_stats, Opts};

fn mock_net_dev_data() -> String {
    "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 11738832  105207    0    0    0     0          0         0 11738832  105207    0    0    0     0       0          0
  eth0: 995608711  460367    0    0    0     0          0         0 32726793  311286    0    0    0     0       0          0
".to_string()
}

fn get_mock_net_dev_stats_from_str(data: &str) -> Result<HashMap<String, (u64, u64)>, io::Error> {
    let reader = BufReader::new(data.as_bytes());
    get_net_dev_stats(reader)
}

#[test]
fn test_edge_cases() {
    // No /proc/net/dev file (simulate by using empty data)
    let result = get_mock_net_dev_stats_from_str("");
    assert!(result.unwrap().is_empty(), "Expected empty stats for empty /proc/net/dev");

    // No network interfaces
    let no_interfaces_data = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
";
    let result = get_mock_net_dev_stats_from_str(no_interfaces_data);
    assert!(result.unwrap().is_empty(), "Expected empty stats for no network interfaces");

    // Corrupt /proc/net/dev data
    let corrupt_data = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: xxxxxxxx  xxxxxx    0    0    0     0          0         0 xxxxxxxx  xxxxxx    0    0    0     0       0          0
  eth0: xxxxxxxx  xxxxxx    0    0    0     0          0         0 xxxxxxxx  xxxxxx    0    0    0     0       0          0
";
    let result = get_mock_net_dev_stats_from_str(corrupt_data);
    assert!(result.is_err(), "Expected error for corrupt /proc/net/dev data");

    // /proc/net/dev with letters instead of numbers
    let letters_data = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: abcdefgh  abcdef    0    0    0     0          0         0 abcdefgh  abcdef    0    0    0     0       0          0
  eth0: abcdefgh  abcdef    0    0    0     0          0         0 abcdefgh  abcdef    0    0    0     0       0          0
";
    let result = get_mock_net_dev_stats_from_str(letters_data);
    assert!(result.is_err(), "Expected error for /proc/net/dev with letters instead of numbers");

    // Empty /proc/net/dev data
    let empty_data = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
";
    let result = get_mock_net_dev_stats_from_str(empty_data);
    assert!(result.unwrap().is_empty(), "Expected empty stats for empty /proc/net/dev");
}

#[test]
fn test_parse_net_dev_stats() {
    let stats = get_mock_net_dev_stats_from_str(&mock_net_dev_data()).unwrap();
    assert_eq!(stats["lo"], (11738832, 11738832));
    assert_eq!(stats["eth0"], (995608711, 32726793));
}

#[test]
fn test_print_headers() {
    let interfaces = vec!["lo".to_string(), "eth0".to_string()];
    let expected = "\
________lo_________________eth0_______
_KB/s_in__KB/s_out___KB/s_in__KB/s_out
";
    let mut output = Vec::new();
    {
        let mut writer = std::io::BufWriter::new(&mut output);
        let stats = get_mock_net_dev_stats_from_str(&mock_net_dev_data()).unwrap();
        print_headers(&interfaces, &mut writer, false, &stats).unwrap();
    }
    let output_str = String::from_utf8(output).unwrap().replace(' ', "_");
    assert_eq!(output_str, expected);
}

#[test]
fn test_print_stats() {
    let previous_stats = get_mock_net_dev_stats_from_str(&mock_net_dev_data()).unwrap();
    let current_stats = get_mock_net_dev_stats_from_str(&mock_net_dev_data()).unwrap();
    let interfaces = vec!["lo".to_string(), "eth0".to_string()];
    let expected = "\
____0.00______0.00______0.00______0.00
";
    let mut output = Vec::new();
    {
        let mut writer = std::io::BufWriter::new(&mut output);
        print_stats(&previous_stats, &current_stats, &interfaces, &mut writer, false).unwrap();
    }
    let output_str = String::from_utf8(output).unwrap().replace(' ', "_");
    assert_eq!(output_str, expected);
}

#[test]
fn test_print_stats_difference() {
    let previous_stats = get_mock_net_dev_stats_from_str(&mock_net_dev_data()).unwrap();
    let new_mock_data = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 11738833  105208    0    0    0     0          0         0 11738834  105208    0    0    0     0       0          0
  eth0: 995708711  460467    0    0    0     0          0         0 32736793  311386    0    0    0     0       0          0
".to_string();
    
    let new_stats = get_mock_net_dev_stats_from_str(&new_mock_data).unwrap();

    let interfaces = vec!["lo".to_string(), "eth0".to_string()];
    let expected = "\
____0.00______0.00_____97.66______9.77
";
    let mut output = Vec::new();
    {
        let mut writer = std::io::BufWriter::new(&mut output);
        print_stats(&previous_stats, &new_stats, &interfaces, &mut writer, false).unwrap();
    }
    let output_str = String::from_utf8(output).unwrap().replace(' ', "_");
    assert_eq!(output_str, expected);
}

#[test]
fn test_command_line_options() {
    let opts = Opts::try_parse_from(&[
        "test",
        "-i",
        "lo,eth0",
        "--first-measurement",
        "0.5",
        "1.0",
        "10",
    ]).unwrap();
    assert_eq!(opts.interfaces.unwrap(), "lo,eth0");
    assert_eq!(opts.first_measurement.unwrap(), 0.5);
    assert_eq!(opts.delay, 1.0);
    assert_eq!(opts.count.unwrap(), 10);
}
