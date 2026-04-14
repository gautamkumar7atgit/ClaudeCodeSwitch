use anyhow::{Context, Result};
use std::process::Command;

use crate::output;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_API_URL: &str =
    "https://api.github.com/repos/gautamkumar7atgit/ClaudeCodeSwitch/releases/latest";
const RELEASE_BASE: &str =
    "https://github.com/gautamkumar7atgit/ClaudeCodeSwitch/releases/download";

pub fn run(verbose: bool) -> Result<()> {
    let exe = std::env::current_exe().context("failed to resolve current binary path")?;
    let exe_str = exe.to_string_lossy();

    // Homebrew-managed binary lives under .../Cellar/...
    if exe_str.contains("Cellar") || exe_str.contains("homebrew") || exe_str.contains("Homebrew") {
        output::print_info("ccswitch is managed by Homebrew.");
        output::print_info("Running: brew upgrade ccswitch");

        let status = Command::new("brew")
            .args(["upgrade", "ccswitch"])
            .status()
            .context("failed to run brew")?;

        if !status.success() {
            anyhow::bail!("brew upgrade failed");
        }
        return Ok(());
    }

    // curl / manual install — self-update
    self_update(&exe, verbose)
}

fn self_update(exe: &std::path::Path, verbose: bool) -> Result<()> {
    output::print_info("Checking for updates...");

    // Fetch latest release tag from GitHub API
    let api_out = Command::new("curl")
        .args([
            "-fsSL",
            "--user-agent",
            "ccswitch-updater",
            GITHUB_API_URL,
        ])
        .output()
        .context("failed to reach GitHub API (check your internet connection)")?;

    if !api_out.status.success() {
        anyhow::bail!("GitHub API request failed");
    }

    let json: serde_json::Value =
        serde_json::from_slice(&api_out.stdout).context("failed to parse GitHub API response")?;

    let latest_tag = json["tag_name"]
        .as_str()
        .context("unexpected GitHub API response — missing tag_name")?;

    let latest_version = latest_tag.trim_start_matches('v');

    if latest_version == CURRENT_VERSION {
        output::print_success(&format!("Already up to date (v{CURRENT_VERSION})"));
        return Ok(());
    }

    output::print_info(&format!(
        "New version available: v{CURRENT_VERSION} → v{latest_version}"
    ));

    let binary_url = format!("{RELEASE_BASE}/{latest_tag}/ccswitch");
    let sha256_url = format!("{RELEASE_BASE}/{latest_tag}/ccswitch.sha256");
    let tmp = "/tmp/ccswitch-update";

    // Download binary
    output::print_info("Downloading...");
    let dl = Command::new("curl")
        .args(["-fsSL", "--progress-bar", "-o", tmp, &binary_url])
        .status()
        .context("curl download failed")?;
    if !dl.success() {
        anyhow::bail!("failed to download binary from {binary_url}");
    }

    // Fetch expected SHA256
    let sha_out = Command::new("curl")
        .args(["-fsSL", "--user-agent", "ccswitch-updater", &sha256_url])
        .output()
        .context("failed to download SHA256 file")?;
    if !sha_out.status.success() {
        let _ = std::fs::remove_file(tmp);
        anyhow::bail!("failed to fetch SHA256 from {sha256_url}");
    }
    let expected = std::str::from_utf8(&sha_out.stdout)
        .context("SHA256 file is not valid UTF-8")?
        .split_whitespace()
        .next()
        .context("SHA256 file is empty")?
        .to_string();

    // Verify SHA256
    let verify = Command::new("shasum")
        .args(["-a", "256", tmp])
        .output()
        .context("failed to run shasum")?;
    let actual = std::str::from_utf8(&verify.stdout)
        .unwrap_or("")
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();

    if actual != expected {
        let _ = std::fs::remove_file(tmp);
        anyhow::bail!("SHA256 mismatch — download may be corrupted\n  expected: {expected}\n  got:      {actual}");
    }

    if verbose {
        output::print_verbose(&format!("SHA256 verified: {actual}"), verbose);
    }

    // Make executable
    Command::new("chmod")
        .args(["+x", tmp])
        .status()
        .context("chmod failed")?;

    // Replace binary — try direct rename first, then sudo cp
    if std::fs::rename(tmp, exe).is_err() {
        output::print_info("Elevated permissions required — you may be prompted for your password.");
        let status = Command::new("sudo")
            .args(["cp", tmp, exe.to_str().unwrap()])
            .status()
            .context("sudo cp failed")?;
        let _ = std::fs::remove_file(tmp);
        if !status.success() {
            anyhow::bail!("failed to replace binary — try running with sudo");
        }
    }

    output::print_success(&format!("Updated to v{latest_version}"));
    Ok(())
}
