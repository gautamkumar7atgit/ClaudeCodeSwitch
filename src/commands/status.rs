use anyhow::Result;
use std::fs;

use crate::config::paths;
use crate::daemon::launchd;
use crate::keychain::{credential_fingerprint, read_keychain};
use crate::output::{bold, cyan, dim, green, print_warn, red, yellow};
use crate::profiles::{get_active_profile, load_profile};

pub fn run(_verbose: bool) -> Result<()> {
    let active_name = get_active_profile()?;

    let Some(name) = active_name else {
        print_warn("No active profile set. Run `ccswitch use <name>` to activate one.");
        return Ok(());
    };

    let profile = load_profile(&name)?;

    // Keychain comparison
    let keychain_match = match read_keychain() {
        Ok(kc) => {
            let matches = kc.claude_ai_oauth.access_token == profile.claude_ai_oauth.access_token;
            let fingerprint = credential_fingerprint(&kc.claude_ai_oauth);
            (true, matches, fingerprint)
        }
        Err(_) => (false, false, String::from("(unreadable)")),
    };

    let profile_fingerprint = credential_fingerprint(&profile.claude_ai_oauth);

    // Daemon status
    let (daemon_status, daemon_pid) = if launchd::daemon_is_loaded() {
        let pid = launchd::get_daemon_pid()
            .map(|p| format!("PID {p}"))
            .unwrap_or_else(|| "running".to_string());
        (format!("running ({pid})"), true)
    } else {
        ("stopped".to_string(), false)
    };

    // Log tail
    let p = paths();
    let log_tail = if p.daemon_log.exists() {
        let content = fs::read_to_string(&p.daemon_log).unwrap_or_default();
        let lines: Vec<String> = content
            .lines()
            .rev()
            .take(3)
            .map(localize_log_timestamp)
            .collect();
        if lines.is_empty() {
            "(empty)".to_string()
        } else {
            lines.into_iter().rev().collect::<Vec<_>>().join("\n       ")
        }
    } else {
        "(no log file)".to_string()
    };

    let match_symbol = if keychain_match.1 {
        green("✓ yes")
    } else {
        red("✗ no")
    };
    let kc_readable = if keychain_match.0 {
        keychain_match.2.clone()
    } else {
        red("(error reading keychain)")
    };
    let _ = daemon_pid;

    let daemon_colored = if daemon_pid {
        green(&daemon_status)
    } else {
        yellow(&daemon_status)
    };

    let token_profile = cyan(&profile_fingerprint);
    let token_keychain = if keychain_match.0 {
        if keychain_match.1 {
            green(&kc_readable)
        } else {
            red(&kc_readable)
        }
    } else {
        kc_readable
    };

    let label = |s: &str| bold(s);

    println!("{} {}", label("Active profile :"), cyan(name.as_str()));
    println!("{} {}", label("Daemon         :"), daemon_colored);
    let last_synced_local = chrono::DateTime::<chrono::Local>::from(profile.meta.last_synced);
    println!("{} {}", label("Last sync      :"), dim(&last_synced_local.format("%Y-%m-%d %H:%M:%S %z").to_string()));
    println!("{} {}", label("Keychain match :"), match_symbol);
    println!(
        "{} {} {}  {} {}",
        label("Access token   :"),
        token_profile,
        dim("(profile)"),
        token_keychain,
        dim("(keychain)")
    );
    println!("{} {}", label("Log tail       :"), dim(&log_tail));

    Ok(())
}

/// Rewrite the leading UTC timestamp in a daemon log line to local time.
/// Log format: `2026-04-08T11:47:35Z [LEVEL] message`
fn localize_log_timestamp(line: &str) -> String {
    // Timestamp is exactly 20 chars: "2026-04-08T11:47:35Z"
    if line.len() >= 20 {
        if let Ok(utc) = chrono::DateTime::parse_from_rfc3339(&line[..20]) {
            let local: chrono::DateTime<chrono::Local> = utc.into();
            return format!("{}{}", local.format("%Y-%m-%d %H:%M:%S %z"), &line[20..]);
        }
    }
    line.to_string()
}
