#[cfg(test)]
mod command_line_options_tests {
    use clap::Parser;
    use ifstat_rs::opts::Opts;

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
}
