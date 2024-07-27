use clap::Parser;
use ifstat_rs::{parse_net_dev_stats, print_headers, print_stats, Opts};
use std::collections::HashMap;
use std::io::Cursor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_net_dev_stats() {
        let data = r#"
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
  eth0:  104013    1264    0    0    0     0          0         0   204386    1571    0    0    0     0       0          0
    lo:  104013    1264    0    0    0     0          0         0   204386    1571    0    0    0     0       0          0
"#;
        println!("Testing parse_net_dev_stats with data:\n{}", data);
        let reader = Cursor::new(data);
        let stats = parse_net_dev_stats(reader).unwrap();

        assert_eq!(stats.len(), 2);
        assert_eq!(stats["eth0"], (104013, 204386));
        assert_eq!(stats["lo"], (104013, 204386));
    }
}

#[test]
fn test_print_headers() {
    let stats = HashMap::new();
    let interfaces = vec!["eth0".to_string(), "lo".to_string()];
    let mut output = Vec::new();
    print_headers(&interfaces, &mut output, false, &stats).unwrap();
    let output_str = String::from_utf8(output).unwrap();

    assert!(output_str.contains("eth0"));
    assert!(output_str.contains("lo"));
}

#[test]
fn test_print_stats() {
    let previous = vec![
        ("eth0".to_string(), (1000, 2000)),
        ("lo".to_string(), (1000, 2000)),
    ]
    .into_iter()
    .collect::<HashMap<_, _>>();
    let current = vec![
        ("eth0".to_string(), (2000, 3000)),
        ("lo".to_string(), (2000, 3000)),
    ]
    .into_iter()
    .collect::<HashMap<_, _>>();
    let interfaces = vec!["eth0".to_string(), "lo".to_string()];
    let mut output = Vec::new();
    print_stats(&previous, &current, &interfaces, &mut output, false).unwrap();
    let output_str = String::from_utf8(output).unwrap();

    let lines: Vec<&str> = output_str.trim().split('\n').collect();

    // We expect one line of output for the two interfaces
    assert_eq!(lines.len(), 1);

    let eth0_lo_output = lines[0];
    let eth0_lo_values: Vec<&str> = eth0_lo_output.split_whitespace().collect();

    // We expect four values (two for each interface: "KB/s in" and "KB/s out")
    assert_eq!(eth0_lo_values.len(), 4);

    let eth0_in: f64 = eth0_lo_values[0].parse().unwrap();
    let eth0_out: f64 = eth0_lo_values[1].parse().unwrap();
    let lo_in: f64 = eth0_lo_values[2].parse().unwrap();
    let lo_out: f64 = eth0_lo_values[3].parse().unwrap();

    let tolerance = 0.01;

    // Calculate the expected values rounded to 2 decimal places
    let expected_eth0_in = (1000.0f64 / 1024.0 * 100.0).round() / 100.0;
    let expected_eth0_out = (1000.0f64 / 1024.0 * 100.0).round() / 100.0;
    let expected_lo_in = (1000.0f64 / 1024.0 * 100.0).round() / 100.0;
    let expected_lo_out = (1000.0f64 / 1024.0 * 100.0).round() / 100.0;

    // Debug output to see the values
    println!("eth0_in: {}, expected: {}", eth0_in, expected_eth0_in);
    println!("eth0_out: {}, expected: {}", eth0_out, expected_eth0_out);
    println!("lo_in: {}, expected: {}", lo_in, expected_lo_in);
    println!("lo_out: {}, expected: {}", lo_out, expected_lo_out);

    // Validate the values with a tolerance
    assert!((eth0_in - expected_eth0_in).abs() < tolerance);
    assert!((eth0_out - expected_eth0_out).abs() < tolerance);
    assert!((lo_in - expected_lo_in).abs() < tolerance);
    assert!((lo_out - expected_lo_out).abs() < tolerance);
}

#[test]
fn test_command_line_options() {
    let opts = Opts::parse_from(&[
        "ifstat-rs",
        "-i",
        "eth0,lo",
        "-a",
        "-l",
        "-z",
        "--first-measurement",
        "3", // First measurement
        "2", // Delay
        "5", // Count
    ]);
    assert_eq!(opts.interfaces, Some("eth0,lo".to_string()));
    assert!(opts.monitor_all);
    assert!(opts.monitor_loopback);
    assert!(opts.hide_zero_counters);
    assert_eq!(opts.delay, 2.0);
    assert_eq!(opts.first_measurement, Some(3.0));
    assert_eq!(opts.count, Some(5));
}

#[test]
fn test_edge_cases() {
    let data = r#"
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
  eth0:  invalid_data
"#;
    let reader = Cursor::new(data);
    let result = parse_net_dev_stats(reader);

    assert!(result.is_err());
}
