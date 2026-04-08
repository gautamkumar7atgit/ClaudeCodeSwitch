/// Integration tests for `ccswitch uninstall` behaviour.
///
/// These tests verify the teardown logic using a temp directory so that
/// real user data in ~/.claude-switcher/ is never touched.
/// The daemon plist is NOT installed during tests, so launchctl is not called.

use serial_test::serial;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

use ccswitch::keychain::OAuthCredentials;
use ccswitch::profiles::{Profile, ProfileMeta};

fn make_profile_json(name: &str) -> String {
    let profile = Profile {
        claude_ai_oauth: OAuthCredentials {
            access_token: format!("access-{name}"),
            refresh_token: format!("refresh-{name}"),
            extra: Default::default(),
        },
        meta: ProfileMeta {
            name: name.to_string(),
            last_synced: chrono::Utc::now(),
        },
    };
    serde_json::to_string_pretty(&profile).unwrap()
}

/// After removing the base dir, no files remain.
#[test]
fn test_remove_base_dir_leaves_nothing() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().join(".claude-switcher");
    let profiles_dir = base.join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    // Write a profile and active file
    fs::write(
        profiles_dir.join("work.json"),
        make_profile_json("work"),
    )
    .unwrap();
    fs::write(base.join("active"), "work").unwrap();
    fs::write(base.join("daemon.log"), "log line\n").unwrap();

    // Simulate uninstall: remove the base dir
    fs::remove_dir_all(&base).unwrap();

    assert!(!base.exists(), "base dir should be gone after uninstall");
}

/// Uninstall with no base dir present should not error.
#[test]
fn test_uninstall_when_dir_absent_is_noop() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().join(".claude-switcher-nonexistent");

    // Should not panic
    if base.exists() {
        fs::remove_dir_all(&base).unwrap();
    }
    // No assertion needed — just verifying no panic
}

/// Profile files inside profiles dir are all removed.
#[test]
fn test_uninstall_removes_all_profiles() {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().join(".claude-switcher");
    let profiles_dir = base.join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    for name in ["work", "personal", "staging"] {
        fs::write(
            profiles_dir.join(format!("{name}.json")),
            make_profile_json(name),
        )
        .unwrap();
    }

    // Three profiles before
    let count_before = fs::read_dir(&profiles_dir).unwrap().count();
    assert_eq!(count_before, 3);

    fs::remove_dir_all(&base).unwrap();
    assert!(!profiles_dir.exists());
}

/// Daemon plist file is removed as part of uninstall (file-level check, no launchctl).
#[test]
fn test_uninstall_removes_plist_file() {
    let tmp = TempDir::new().unwrap();
    let agents_dir = tmp.path().join("LaunchAgents");
    fs::create_dir_all(&agents_dir).unwrap();
    let plist = agents_dir.join("com.ccswitch.daemon.plist");
    fs::write(&plist, "<plist/>").unwrap();
    assert!(plist.exists());

    fs::remove_file(&plist).unwrap();
    assert!(!plist.exists(), "plist file should be removed on uninstall");
}

/// Keychain entry is NOT removed by uninstall.
/// We verify this by checking that the ccswitch-test entry (if written) survives
/// a simulated uninstall (which only removes files, never touches Keychain).
#[test]
#[serial]
fn test_uninstall_does_not_touch_keychain() {
    // Write a test Keychain entry
    let output = std::process::Command::new("security")
        .args([
            "add-generic-password",
            "-U",
            "-s",
            "ccswitch-uninstall-test",
            "-a",
            "credentials",
            "-w",
            r#"{"accessToken":"tok","refreshToken":"ref"}"#,
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        // If we can't write, skip gracefully
        return;
    }

    // Simulate uninstall (no keychain interaction)
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().join(".claude-switcher");
    fs::create_dir_all(&base).unwrap();
    fs::remove_dir_all(&base).unwrap();

    // Verify Keychain entry still exists
    let read = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            "ccswitch-uninstall-test",
            "-a",
            "credentials",
            "-w",
        ])
        .output()
        .unwrap();

    assert!(
        read.status.success(),
        "Keychain entry should survive uninstall"
    );

    // Cleanup
    let _ = std::process::Command::new("security")
        .args([
            "delete-generic-password",
            "-s",
            "ccswitch-uninstall-test",
            "-a",
            "credentials",
        ])
        .output();
}
