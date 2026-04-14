use anyhow::{Context, Result};
use std::process::Command;

use crate::output;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_API_URL: &str =
    "https://api.github.com/repos/gautamkumar7atgit/ClaudeCodeSwitch/releases/latest";
const RELEASE_BASE: &str =
    "https://github.com/gautamkumar7atgit/ClaudeCodeSwitch/releases/download";
const TAP_NAME: &str = "gautamkumar7atgit/ccswitch";

pub fn run(verbose: bool) -> Result<()> {
    let exe = std::env::current_exe().context("failed to resolve current binary path")?;
    let exe_str = exe.to_string_lossy();

    // Homebrew-managed binary lives under .../Cellar/...
    if exe_str.contains("Cellar") || exe_str.contains("homebrew") || exe_str.contains("Homebrew") {
        return homebrew_update(verbose);
    }

    // curl / manual install — self-update
    self_update(&exe, verbose)
}

// ── Homebrew path ─────────────────────────────────────────────────────────────

fn homebrew_update(verbose: bool) -> Result<()> {
    output::print_info("ccswitch is managed by Homebrew.");

    // Check GitHub for the real latest version first
    let latest_tag = fetch_latest_tag()?;
    let latest_version = latest_tag.trim_start_matches('v');

    if latest_version == CURRENT_VERSION {
        output::print_success(&format!("Already up to date (v{CURRENT_VERSION})"));
        return Ok(());
    }

    output::print_info(&format!(
        "New version available: v{CURRENT_VERSION} → v{latest_version}"
    ));

    // Try a normal brew upgrade first
    output::print_info("Running: brew upgrade ccswitch");
    let out = Command::new("brew")
        .args(["upgrade", "ccswitch"])
        .output()
        .context("failed to run brew")?;

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    if verbose {
        output::print_verbose(&combined, verbose);
    }

    // brew prints "already installed" when its local tap cache is stale
    // (user hasn't run --force-auto-update). Detect and recover automatically.
    if combined.contains("already installed") {
        output::print_warn(
            "Homebrew's tap cache is stale — the formula hasn't been refreshed yet.",
        );
        output::print_info("Refreshing tap automatically...");

        untap_and_retap(verbose)?;

        output::print_info("Running: brew upgrade ccswitch");
        let status = Command::new("brew")
            .args(["upgrade", "ccswitch"])
            .status()
            .context("failed to run brew upgrade after retap")?;

        if !status.success() {
            anyhow::bail!("brew upgrade failed after tap refresh");
        }

        output::print_info(
            "Tip: run `brew tap --force-auto-update gautamkumar7atgit/ccswitch` once \
             to keep the tap auto-updated in the future.",
        );
        return Ok(());
    }

    if !out.status.success() {
        anyhow::bail!("brew upgrade failed");
    }

    output::print_success(&format!("Updated to v{latest_version}"));
    Ok(())
}

fn untap_and_retap(verbose: bool) -> Result<()> {
    output::print_info(&format!("Running: brew untap {TAP_NAME}"));
    let s1 = Command::new("brew")
        .args(["untap", TAP_NAME])
        .status()
        .context("brew untap failed")?;
    if !s1.success() {
        anyhow::bail!("brew untap {TAP_NAME} failed");
    }

    output::print_info(&format!("Running: brew tap {TAP_NAME}"));
    let s2 = Command::new("brew")
        .args(["tap", TAP_NAME])
        .status()
        .context("brew tap failed")?;
    if !s2.success() {
        anyhow::bail!("brew tap {TAP_NAME} failed");
    }

    if verbose {
        output::print_verbose("Tap refreshed successfully.", verbose);
    }
    Ok(())
}

// ── curl / self-update path ───────────────────────────────────────────────────

fn self_update(exe: &std::path::Path, verbose: bool) -> Result<()> {
    output::print_info("Checking for updates...");

    let latest_tag = fetch_latest_tag()?;
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
        anyhow::bail!(
            "SHA256 mismatch — download may be corrupted\n  expected: {expected}\n  got:      {actual}"
        );
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
        output::print_info(
            "Elevated permissions required — you may be prompted for your password.",
        );
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

// ── Shared helpers ────────────────────────────────────────────────────────────

fn fetch_latest_tag() -> Result<String> {
    let out = Command::new("curl")
        .args([
            "-fsSL",
            "--user-agent",
            "ccswitch-updater",
            GITHUB_API_URL,
        ])
        .output()
        .context("failed to reach GitHub API (check your internet connection)")?;

    if !out.status.success() {
        anyhow::bail!("GitHub API request failed");
    }

    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).context("failed to parse GitHub API response")?;

    json["tag_name"]
        .as_str()
        .context("unexpected GitHub API response — missing tag_name")
        .map(|s| s.to_string())
}
