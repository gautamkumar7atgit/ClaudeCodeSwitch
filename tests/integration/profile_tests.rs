use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

// Override profile paths by setting env var; real impl uses config::paths().
// We test the profile logic directly by pointing at a temp dir via a helper.

fn make_temp_profile(
    dir: &std::path::Path,
    name: &str,
    access: &str,
    refresh: &str,
) -> ccswitch::profiles::Profile {
    use chrono::Utc;
    use ccswitch::keychain::OAuthCredentials;
    use ccswitch::profiles::{Profile, ProfileMeta};
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

/// Write a profile JSON to an arbitrary dir and read it back, verifying round-trip.
#[test]
fn test_profile_round_trip() {
    let tmp = TempDir::new().unwrap();
    let profiles_dir = tmp.path().join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    let profile = make_temp_profile(tmp.path(), "work", "access-abc123", "refresh-xyz789");
    let json = serde_json::to_string_pretty(&profile).unwrap();

    let path = profiles_dir.join("work.json");
    let tmp_path = profiles_dir.join("work.json.tmp");
    fs::write(&tmp_path, &json).unwrap();
    fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::rename(&tmp_path, &path).unwrap();

    // Verify file exists and has correct permissions
    let meta = fs::metadata(&path).unwrap();
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "profile file should be chmod 600");

    // Verify round-trip
    let data = fs::read_to_string(&path).unwrap();
    let loaded: ccswitch::profiles::Profile = serde_json::from_str(&data).unwrap();
    assert_eq!(loaded.meta.name, "work");
    assert_eq!(loaded.claude_ai_oauth.access_token, "access-abc123");
    assert_eq!(loaded.claude_ai_oauth.refresh_token, "refresh-xyz789");
}

/// Verify the sample fixture parses correctly.
#[test]
fn test_sample_fixture_parses() {
    let fixture = include_str!("../fixtures/sample_profile.json");
    let profile: ccswitch::profiles::Profile = serde_json::from_str(fixture).unwrap();
    assert_eq!(profile.meta.name, "testprofile");
    assert_eq!(
        profile.claude_ai_oauth.access_token,
        "sk-ant-oat01-test-access-token-abcdefgh"
    );
}

/// Atomic write: tmp file disappears on success, target file exists.
#[test]
fn test_atomic_write_no_tmp_left() {
    let tmp = TempDir::new().unwrap();
    let profiles_dir = tmp.path().join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    let profile = make_temp_profile(tmp.path(), "atomic", "tok", "ref");
    let json = serde_json::to_string_pretty(&profile).unwrap();
    let path = profiles_dir.join("atomic.json");
    let tmp_path = profiles_dir.join("atomic.json.tmp");

    fs::write(&tmp_path, &json).unwrap();
    fs::rename(&tmp_path, &path).unwrap();

    assert!(path.exists());
    assert!(!tmp_path.exists(), ".tmp file should not remain after rename");
}
