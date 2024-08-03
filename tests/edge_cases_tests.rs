#[cfg(target_os = "linux")]
mod edge_cases_tests {
    use ifstat_rs::net_stats::parse_net_dev_stats;
    use std::io::Cursor;

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
}
