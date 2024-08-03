#[cfg(target_os = "linux")]
mod parse_net_dev_stats_tests {
    use ifstat_rs::net_stats::parse_net_dev_stats;
    use std::io::Cursor;

    #[test]
    fn test_parse_net_dev_stats() {
        let data = r#"Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
   lo:  104013    1264    0    0    0     0          0         0   204386    1571    0    0    0     0       0          0
 eth0:  104013    1264    0    0    0     0          0         0   204386    1571    0    0    0     0       0          0
"#;
        let reader = Cursor::new(data);
        let stats = parse_net_dev_stats(reader).unwrap();

        assert_eq!(stats.len(), 2);
        assert_eq!(stats["eth0"], (104013, 204386));
        assert_eq!(stats["lo"], (104013, 204386));
    }
}
