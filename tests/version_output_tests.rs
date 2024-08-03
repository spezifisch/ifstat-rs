mod version_output_tests {
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::process::Command;

    #[test]
    fn test_version_output() {
        let mut cmd = Command::cargo_bin("ifstat-rs").unwrap();
        cmd.arg("-V")
            .assert()
            .stdout(predicate::str::is_match(r"ifstat-rs \d+\.\d+\.\d+\n").unwrap());
    }
}
