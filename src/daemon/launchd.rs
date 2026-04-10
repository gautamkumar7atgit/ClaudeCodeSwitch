use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::config::paths;

const LABEL: &str = "com.ccswitch.daemon";

pub fn generate_plist(binary_path: &Path) -> String {
    let binary = binary_path.display();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{binary}</string>
        <string>--daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{log}</string>
    <key>StandardErrorPath</key>
    <string>{log}</string>
</dict>
</plist>
"#,
        log = paths().daemon_log.display()
    )
}

fn gui_domain() -> String {
    // SAFETY: getuid() is always safe to call
    format!("gui/{}", unsafe { libc::getuid() })
}

pub fn install_plist(binary_path: &Path) -> Result<()> {
    let p = paths();
    let plist_content = generate_plist(binary_path);

    // Ensure LaunchAgents dir exists
    fs::create_dir_all(&p.launch_agents_dir)
        .with_context(|| format!("failed to create {}", p.launch_agents_dir.display()))?;

    fs::write(&p.plist_file, &plist_content)
        .with_context(|| format!("failed to write plist to {}", p.plist_file.display()))?;

    // Use bootstrap (macOS 13+); plist stays on disk so it auto-loads on every login
    let output = Command::new("launchctl")
        .args(["bootstrap", &gui_domain(), &p.plist_file.to_string_lossy()])
        .output()
        .context("failed to run launchctl bootstrap")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("launchctl bootstrap failed: {stderr}");
    }

    Ok(())
}

/// Stop the running daemon without removing the plist — it will auto-start on next login.
pub fn stop_daemon() -> Result<()> {
    let domain_target = format!("{}/{}", gui_domain(), LABEL);
    let output = Command::new("launchctl")
        .args(["bootout", &domain_target])
        .output()
        .context("failed to run launchctl bootout")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("launchctl bootout failed: {stderr}");
    }

    Ok(())
}

/// Stop the daemon AND remove the plist (full uninstall — won't auto-start on login).
pub fn uninstall_plist() -> Result<()> {
    let p = paths();

    if daemon_is_loaded() {
        stop_daemon()?;
    }

    if p.plist_file.exists() {
        fs::remove_file(&p.plist_file)
            .with_context(|| format!("failed to remove {}", p.plist_file.display()))?;
    }

    Ok(())
}

pub fn daemon_is_loaded() -> bool {
    Command::new("launchctl")
        .args(["list", LABEL])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_daemon_pid() -> Option<u32> {
    let output = Command::new("launchctl")
        .args(["list", LABEL])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // launchctl list output: "PID\tStatus\tLabel"
    // First line is a header, second is the entry
    let stdout = String::from_utf8(output.stdout).ok()?;
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 && parts[2].contains(LABEL) {
            return parts[0].trim().parse::<u32>().ok();
        }
    }

    None
}
