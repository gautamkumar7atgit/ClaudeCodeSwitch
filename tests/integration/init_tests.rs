/// Integration tests for `ccswitch init` behaviour.
///
/// These tests exercise the underlying building blocks of the init command
/// (ensure_dirs, save_profile, set_active_profile) without touching the
/// real daemon or issuing interactive prompts.

use serial_test::serial;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use ccswitch::config::paths;
use ccswitch::keychain::OAuthCredentials;
use ccswitch::profiles::{
    clear_active_profile, delete_profile, ensure_dirs, get_active_profile, save_profile,
    set_active_profile, Profile, ProfileMeta,
};

fn make_profile(name: &str) -> Profile {
    Profile {
        claude_ai_oauth: OAuthCredentials {
            access_token: format!("access-{name}"),
            refresh_token: format!("refresh-{name}"),
            extra: Default::default(),
        },
        meta: ProfileMeta {
            name: name.to_string(),
            last_synced: chrono::Utc::now(),
        },
    }
}

/// ensure_dirs() creates ~/.claude-switcher/profiles/ with correct permissions.
#[test]
#[serial]
fn test_ensure_dirs_creates_structure() {
    ensure_dirs().unwrap();

    let p = paths();
    assert!(
        p.profiles_dir.exists(),
        "profiles dir should exist after ensure_dirs"
    );

    // Base dir should be chmod 700
    let base = p.profiles_dir.parent().unwrap();
    let mode = fs::metadata(base).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o700, "base dir should be chmod 700");
}

/// ensure_dirs() is idempotent — calling it twice does not error.
#[test]
#[serial]
fn test_ensure_dirs_idempotent() {
    ensure_dirs().unwrap();
    ensure_dirs().unwrap(); // second call must not fail
}

/// Fresh-install flow: ensure_dirs → save profile → set active → verify.
#[test]
#[serial]
fn test_init_flow_creates_profile_and_sets_active() {
    let name = "init-test-profile";

    ensure_dirs().unwrap();
    let profile = make_profile(name);
    save_profile(&profile).unwrap();
    set_active_profile(name).unwrap();

    // Verify active file contents
    let active = get_active_profile().unwrap();
    assert_eq!(active.as_deref(), Some(name));

    // Verify profile file exists and is chmod 600
    let p = paths();
    let profile_path = p.profiles_dir.join(format!("{name}.json"));
    assert!(profile_path.exists());
    let mode = fs::metadata(&profile_path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);

    // Cleanup
    delete_profile(name).unwrap();
    clear_active_profile().unwrap();
}

/// If credentials already exist in profiles dir, re-running ensure_dirs preserves them.
#[test]
#[serial]
fn test_reinit_preserves_existing_profiles() {
    let name = "init-reinit-profile";

    ensure_dirs().unwrap();
    let profile = make_profile(name);
    save_profile(&profile).unwrap();

    // Simulate re-init
    ensure_dirs().unwrap();

    // Profile must still be readable
    let loaded = ccswitch::profiles::load_profile(name).unwrap();
    assert_eq!(loaded.meta.name, name);

    delete_profile(name).unwrap();
}

/// Active file is written atomically (no .tmp left behind after set_active_profile).
#[test]
#[serial]
fn test_active_file_no_tmp_leftover() {
    ensure_dirs().unwrap();
    set_active_profile("no-tmp-test").unwrap();

    let p = paths();
    let tmp = p.active_file.with_extension("tmp");
    assert!(!tmp.exists(), "active.tmp should not remain after set_active_profile");

    clear_active_profile().unwrap();
}

/// get_active_profile returns None when active file is absent.
#[test]
#[serial]
fn test_no_active_profile_when_file_absent() {
    ensure_dirs().unwrap();
    clear_active_profile().unwrap(); // ensure it's gone

    let active = get_active_profile().unwrap();
    assert!(active.is_none());
}
