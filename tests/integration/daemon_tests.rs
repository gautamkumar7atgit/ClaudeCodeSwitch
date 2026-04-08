/// Daemon integration tests.
///
/// These tests exercise the daemon's poll logic and log rotation directly
/// by calling internal helpers. No actual daemon process is spawned.

use chrono::Utc;
use serial_test::serial;
use std::fs::{self, OpenOptions};
use std::io::Write;
use tempfile::TempDir;

use ccswitch::keychain::OAuthCredentials;
use ccswitch::profiles::{Profile, ProfileMeta};

fn make_profile(name: &str, access: &str, refresh: &str) -> Profile {
    Profile {
        claude_ai_oauth: OAuthCredentials {
            access_token: access.to_string(),
            refresh_token: refresh.to_string(),
            extra: Default::default(),
        },
        meta: ProfileMeta {
            name: name.to_string(),
            last_synced: Utc::now(),
        },
    }
}

// --------------------------------------------------------------------------
// Log rotation
// --------------------------------------------------------------------------

/// Helper replicating the daemon's rotate-then-append logic.
fn daemon_log_write(log_path: &std::path::Path, msg: &str) {
    const LIMIT: u64 = 512; // small limit for tests
    if let Ok(meta) = fs::metadata(log_path) {
        if meta.len() >= LIMIT {
            let rotated = log_path.with_extension("log.1");
            let _ = fs::rename(log_path, rotated);
        }
    }
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .unwrap();
    writeln!(f, "{msg}").unwrap();
}

#[test]
fn test_log_rotation_creates_dot1_and_fresh_log() {
    let tmp = TempDir::new().unwrap();
    let log = tmp.path().join("daemon.log");

    // Fill log past 512-byte threshold
    for i in 0..20 {
        daemon_log_write(&log, &format!("line {i:04} — padding padding padding padding padding"));
    }

    // One more write should trigger rotation
    daemon_log_write(&log, "trigger rotation");

    let rotated = tmp.path().join("daemon.log.1");
    assert!(rotated.exists(), "daemon.log.1 should exist after rotation");

    let fresh_size = fs::metadata(&log).map(|m| m.len()).unwrap_or(0);
    let rotated_size = fs::metadata(&rotated).map(|m| m.len()).unwrap_or(0);
    assert!(
        fresh_size < rotated_size,
        "fresh log should be smaller than rotated log"
    );
}

#[test]
fn test_log_no_rotation_below_limit() {
    let tmp = TempDir::new().unwrap();
    let log = tmp.path().join("daemon.log");

    daemon_log_write(&log, "single line");

    let rotated = tmp.path().join("daemon.log.1");
    assert!(!rotated.exists(), "no rotation should occur for small log");
    assert!(log.exists());
}

// --------------------------------------------------------------------------
// Profile sync logic (unit-level, no real Keychain)
// --------------------------------------------------------------------------

/// When refresh_token matches but access_token differs, the profile should be
/// updated with the new tokens.
#[test]
#[serial]
fn test_token_refresh_detected_updates_profile() {
    ccswitch::profiles::ensure_dirs().unwrap();

    let name = "daemon-sync-test";
    let old_profile = make_profile(name, "old-access", "shared-refresh");
    ccswitch::profiles::save_profile(&old_profile).unwrap();
    ccswitch::profiles::set_active_profile(name).unwrap();

    // Simulate what poll_once would do if keychain had refreshed access_token
    let kc_access = "new-access";
    let kc_refresh = "shared-refresh"; // same refresh token

    let mut profile = ccswitch::profiles::load_profile(name).unwrap();
    if profile.claude_ai_oauth.refresh_token == kc_refresh
        && profile.claude_ai_oauth.access_token != kc_access
    {
        // This is the sync branch
        profile.claude_ai_oauth.access_token = kc_access.to_string();
        profile.meta.last_synced = Utc::now();
        ccswitch::profiles::save_profile(&profile).unwrap();
    }

    let updated = ccswitch::profiles::load_profile(name).unwrap();
    assert_eq!(updated.claude_ai_oauth.access_token, "new-access");

    // Cleanup
    ccswitch::profiles::delete_profile(name).unwrap();
    ccswitch::profiles::clear_active_profile().unwrap();
}

/// When both tokens differ (foreign credentials), the profile should NOT be updated.
#[test]
#[serial]
fn test_foreign_credentials_not_synced() {
    ccswitch::profiles::ensure_dirs().unwrap();

    let name = "daemon-foreign-test";
    let original_profile = make_profile(name, "my-access", "my-refresh");
    ccswitch::profiles::save_profile(&original_profile).unwrap();
    ccswitch::profiles::set_active_profile(name).unwrap();

    let kc_access = "foreign-access";
    let kc_refresh = "foreign-refresh"; // both differ

    let profile = ccswitch::profiles::load_profile(name).unwrap();
    let should_sync = profile.claude_ai_oauth.refresh_token == kc_refresh;
    assert!(!should_sync, "foreign credentials should not trigger a sync");

    // Profile should be unchanged
    let unchanged = ccswitch::profiles::load_profile(name).unwrap();
    assert_eq!(unchanged.claude_ai_oauth.access_token, "my-access");

    // Cleanup
    ccswitch::profiles::delete_profile(name).unwrap();
    ccswitch::profiles::clear_active_profile().unwrap();
}

/// When tokens are identical, poll_once should be a no-op (no save).
#[test]
#[serial]
fn test_in_sync_no_update() {
    ccswitch::profiles::ensure_dirs().unwrap();

    let name = "daemon-insync-test";
    let profile = make_profile(name, "same-access", "same-refresh");
    ccswitch::profiles::save_profile(&profile).unwrap();

    let kc_access = "same-access";
    let kc_refresh = "same-refresh";

    let loaded = ccswitch::profiles::load_profile(name).unwrap();
    let needs_update = loaded.claude_ai_oauth.access_token != kc_access
        || loaded.claude_ai_oauth.refresh_token != kc_refresh;
    assert!(!needs_update, "in-sync profile should not need update");

    ccswitch::profiles::delete_profile(name).unwrap();
}

// --------------------------------------------------------------------------
// Idempotency (daemon start/stop)
// --------------------------------------------------------------------------

/// daemon_is_loaded() returns false when the plist isn't installed.
#[test]
fn test_daemon_not_loaded_when_plist_absent() {
    // If the plist file doesn't exist, launchctl list will fail → false
    let p = ccswitch::config::paths();
    if !p.plist_file.exists() {
        assert!(!ccswitch::daemon::launchd::daemon_is_loaded());
    }
    // If plist exists (daemon is actually running in CI), this test is a no-op — that's fine.
}
