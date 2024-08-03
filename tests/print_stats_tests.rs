mod print_stats_tests {
    use ifstat_rs::output::print_stats;
    use indexmap::IndexMap;

    #[test]
    fn test_print_stats() {
        let previous = vec![
            ("eth0".to_string(), (1000, 2000)),
            ("lo".to_string(), (1000, 2000)),
        ]
        .into_iter()
        .collect::<IndexMap<_, _>>();
        let current = vec![
            ("eth0".to_string(), (2000, 3000)),
            ("lo".to_string(), (2000, 3000)),
        ]
        .into_iter()
        .collect::<IndexMap<_, _>>();
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
}
