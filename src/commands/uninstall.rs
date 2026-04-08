use anyhow::Result;
use std::fs;

use crate::config::paths;
use crate::daemon::launchd;
use crate::output::{confirm_prompt, print_success, print_warn};

pub fn run(_verbose: bool) -> Result<()> {
    println!("This will remove:");
    println!("  • ~/.claude-switcher/ (all profiles and state)");
    println!("  • ~/Library/LaunchAgents/com.ccswitch.daemon.plist");
    println!("Keychain credentials will NOT be touched.");
    println!();

    let confirmed = confirm_prompt("Continue?")?;
    if !confirmed {
        return Ok(());
    }

    // Stop daemon
    if launchd::daemon_is_loaded() {
        launchd::uninstall_plist()?;
    } else {
        // Remove plist file if it exists but isn't loaded
        let p = paths();
        if p.plist_file.exists() {
            fs::remove_file(&p.plist_file)?;
        }
    }

    // Remove data dir
    let p = paths();
    let base = p.profiles_dir.parent().unwrap().to_path_buf();
    if base.exists() {
        fs::remove_dir_all(&base)?;
    } else {
        print_warn("~/.claude-switcher/ not found, nothing to remove.");
    }

    print_success("ccswitch uninstalled. Claude Code credentials are intact.");
    Ok(())
}
