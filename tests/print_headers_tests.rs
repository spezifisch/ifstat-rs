mod print_headers_tests {
    use ifstat_rs::output::print_headers;
    use indexmap::IndexMap;

    #[test]
    fn test_print_headers() {
        let stats = IndexMap::new();
        let interfaces = vec!["eth0".to_string(), "lo".to_string()];
        let mut output = Vec::new();
        print_headers(&interfaces, &mut output, false, &stats).unwrap();
        let output_str = String::from_utf8(output).unwrap();

        assert!(output_str.contains("eth0"));
        assert!(output_str.contains("lo"));
    }
}
