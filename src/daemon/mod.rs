pub mod launchd;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{Local, Utc};

use crate::config::{paths, DAEMON_LOG_MAX_BYTES, POLL_INTERVAL_SECS};
use crate::keychain::read_keychain;
use crate::profiles::{get_active_profile, load_profile, save_profile};

pub fn run_daemon() {
    let p = paths();

    // Ensure the data dir exists before we try to open the log
    let _ = crate::profiles::ensure_dirs();

    // Set up SIGTERM handler — sets a flag so the main loop exits cleanly
    let shutdown = Arc::new(AtomicBool::new(false));
    {
        let flag = Arc::clone(&shutdown);
        unsafe {
            let _ = nix::sys::signal::signal(
                nix::sys::signal::Signal::SIGTERM,
                nix::sys::signal::SigHandler::Handler(handle_sigterm),
            );
        }
        // Store the pointer so the handler can reach it
        SHUTDOWN_FLAG.store(Arc::into_raw(flag) as *mut _, Ordering::SeqCst);
    }

    daemon_log(&p.daemon_log, "INFO", "ccswitch daemon started");

    loop {
        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            daemon_log(&p.daemon_log, "INFO", "SIGTERM received — shutting down");
            break;
        }

        // Sleep in 1s increments so SIGTERM is noticed promptly
        for _ in 0..POLL_INTERVAL_SECS {
            std::thread::sleep(Duration::from_secs(1));
            if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
                break;
            }
        }

        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            daemon_log(&p.daemon_log, "INFO", "SIGTERM received — shutting down");
            break;
        }

        if let Err(e) = poll_once() {
            daemon_log(&p.daemon_log, "ERROR", &format!("poll error: {e}"));
        }
    }
}

fn poll_once() -> anyhow::Result<()> {
    let p = paths();

    let active_name = match get_active_profile()? {
        Some(n) => n,
        None => {
            daemon_log(&p.daemon_log, "DEBUG", "no active profile, skipping poll");
            return Ok(());
        }
    };

    let mut profile = load_profile(&active_name)?;

    let kc = match read_keychain() {
        Ok(kc) => kc,
        Err(e) => {
            daemon_log(&p.daemon_log, "WARN", &format!("cannot read keychain: {e}"));
            return Ok(());
        }
    };

    let kc_creds = &kc.claude_ai_oauth;
    let profile_creds = &profile.claude_ai_oauth;

    if kc_creds.access_token == profile_creds.access_token
        && kc_creds.refresh_token == profile_creds.refresh_token
    {
        // In sync — nothing to do
        return Ok(());
    }

    // Keychain differs from profile — Claude Code refreshed tokens (access-only or full
    // rotation). Always sync back so the profile stays current.
    let reason = if kc_creds.refresh_token == profile_creds.refresh_token {
        "access token refresh"
    } else {
        "full token rotation"
    };
    daemon_log(
        &p.daemon_log,
        "INFO",
        &format!("{reason} detected for \"{active_name}\" — syncing"),
    );
    profile.claude_ai_oauth = kc_creds.clone();
    profile.meta.last_synced = Utc::now();
    save_profile(&profile)?;

    Ok(())
}

// --------------------------------------------------------------------------
// Log helpers
// --------------------------------------------------------------------------

fn daemon_log(log_path: &std::path::Path, level: &str, msg: &str) {
    // Rotate if over limit
    if let Ok(meta) = fs::metadata(log_path) {
        if meta.len() >= DAEMON_LOG_MAX_BYTES {
            let rotated = log_path.with_extension("log.1");
            let _ = fs::rename(log_path, rotated);
        }
    }

    let ts = Local::now().format("%Y-%m-%d %H:%M:%S %z");
    let line = format!("{ts} [{level}] {msg}\n");

    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = f.write_all(line.as_bytes());
    }
}

// --------------------------------------------------------------------------
// SIGTERM handling via a global atomic flag
// --------------------------------------------------------------------------

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

// Raw pointer to the Arc — set once at startup, never freed (process exits)
static SHUTDOWN_FLAG: std::sync::atomic::AtomicPtr<()> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

extern "C" fn handle_sigterm(_: libc::c_int) {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}
