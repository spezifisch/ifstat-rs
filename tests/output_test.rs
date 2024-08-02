#[cfg(test)]
mod tests {
    use ifstat_rs::output::print_net_devices;
    use std::collections::HashMap;

    #[test]
    fn test_print_net_devices() {
        let mut stats = HashMap::new();
        stats.insert("lo".to_string(), (0, 0));
        stats.insert("eth0".to_string(), (0, 0));

        let output = capture_stdout(|| {
            print_net_devices(&stats);
        })
        .expect("Failed to capture stdout");

        let expected_output = "2 interfaces:\nlo\neth0\n";
        assert_eq!(output, expected_output);
    }
}
