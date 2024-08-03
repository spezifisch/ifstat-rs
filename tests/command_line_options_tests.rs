#[cfg(test)]
mod command_line_options_tests {
    use clap::Parser;
    use ifstat_rs::opts::Opts;

    #[test]
    fn test_valid_command_line_options() {
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
        assert_eq!(opts.first_measurement, Some(3.0));
        assert_eq!(opts.delay, 2.0);
        assert_eq!(opts.count, Some(5));
    }

    #[test]
    fn test_delay_greater_than_zero() {
        let opts = Opts::parse_from(&[
            "ifstat-rs",
            "--first-measurement",
            "0",
            "0.1", // Delay
            "1",   // Count
        ]);
        assert_eq!(opts.first_measurement, Some(0.0));
        assert_eq!(opts.delay, 0.1);
        assert_eq!(opts.count, Some(1));
    }

    #[test]
    fn test_delay_zero_should_fail() {
        let result = Opts::try_parse_from(&["ifstat-rs", "0"]);
        assert!(result.is_err());
        let error_message = format!("{}", result.err().unwrap());
        assert!(error_message.contains("`0` must be greater than 0"));
    }

    #[test]
    fn test_delay_negative_should_fail() {
        let result = Opts::try_parse_from(&["ifstat-rs", "--", "-1"]);
        assert!(result.is_err());
        let error_message = format!("{}", result.err().unwrap());
        assert!(error_message.contains("`-1` must be greater than 0"));
    }

    #[test]
    fn test_first_measurement_zero() {
        let opts = Opts::parse_from(&[
            "ifstat-rs",
            "--first-measurement=0",
            "1", // Delay
        ]);
        assert_eq!(opts.first_measurement, Some(0.0));
        assert_eq!(opts.delay, 1.0);
    }

    #[test]
    fn test_first_measurement_negative_should_fail() {
        let result = Opts::try_parse_from(&[
            "ifstat-rs",
            "1", // Delay
            "--first-measurement=-1",
        ]);
        assert!(result.is_err());
        let error_message = format!("{}", result.err().unwrap());
        assert!(
            error_message.contains("must be greater than or equal to 0"),
            "errmsg={}",
            error_message
        );
    }

    #[test]
    fn test_count_zero_should_fail() {
        let result = Opts::try_parse_from(&[
            "ifstat-rs",
            "1", // Delay
            "0", // Count
        ]);
        assert!(result.is_err());
        let error_message = format!("{}", result.err().unwrap());
        assert!(error_message.contains("must be greater than 0"));
    }

    #[test]
    fn test_count_negative_unescaped_should_fail() {
        let result = Opts::try_parse_from(&[
            "ifstat-rs",
            "1",  // Delay
            "-1", // Count
        ]);
        assert!(result.is_err());
        let error_message = format!("{}", result.err().unwrap());
        assert!(error_message.contains("unexpected argument '-1'"));
    }

    #[test]
    fn test_count_negative_should_fail() {
        let result = Opts::try_parse_from(&[
            "ifstat-rs",
            "1", // Delay
            "--",
            "-1", // Count
        ]);
        assert!(result.is_err());
        let error_message = format!("{}", result.err().unwrap());
        assert!(error_message.contains("not a valid number > 0"));
    }
}
