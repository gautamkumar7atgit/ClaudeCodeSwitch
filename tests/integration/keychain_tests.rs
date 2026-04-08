/// Keychain integration tests.
///
/// These tests shell out to the real `security` CLI using the test service name
/// `ccswitch-test` — never `Claude Code-credentials`.
/// Each test cleans up after itself.
/// Tests are serialized (via `#[serial]`) to avoid keychain access conflicts.

use serial_test::serial;
use std::process::Command;

const TEST_SERVICE: &str = "ccswitch-test";
const TEST_ACCOUNT: &str = "credentials";

fn delete_test_entry() {
    let _ = Command::new("security")
        .args([
            "delete-generic-password",
            "-s", TEST_SERVICE,
            "-a", TEST_ACCOUNT,
        ])
        .output();
}

fn write_test_entry(json: &str) {
    let output = Command::new("security")
        .args([
            "add-generic-password",
            "-U",
            "-s", TEST_SERVICE,
            "-a", TEST_ACCOUNT,
            "-w", json,
        ])
        .output()
        .expect("failed to run security CLI");
    assert!(output.status.success(), "failed to write test keychain entry");
}

fn read_test_entry() -> String {
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-s", TEST_SERVICE,
            "-a", TEST_ACCOUNT,
            "-w",
        ])
        .output()
        .expect("failed to run security CLI");
    assert!(output.status.success(), "failed to read test keychain entry");
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[test]
#[serial]
fn test_keychain_write_and_read() {
    delete_test_entry();
    let json = r#"{"accessToken":"test-access","refreshToken":"test-refresh"}"#;
    write_test_entry(json);
    let read_back = read_test_entry();
    assert_eq!(read_back, json);
    delete_test_entry();
}

#[test]
#[serial]
fn test_keychain_overwrite() {
    delete_test_entry();
    write_test_entry(r#"{"accessToken":"old","refreshToken":"old-r"}"#);
    write_test_entry(r#"{"accessToken":"new","refreshToken":"new-r"}"#);
    let read_back = read_test_entry();
    assert!(read_back.contains("new"), "overwrite should replace old value");
    delete_test_entry();
}

#[test]
#[serial]
fn test_keychain_not_found_exit_code() {
    delete_test_entry();
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-s", TEST_SERVICE,
            "-a", TEST_ACCOUNT,
            "-w",
        ])
        .output()
        .expect("failed to run security CLI");
    assert_eq!(
        output.status.code(),
        Some(44),
        "missing keychain entry should exit 44"
    );
}
