use anyhow::Result;
use chrono::Utc;

use crate::keychain::read_keychain;
use crate::output::{confirm_prompt, print_info, print_success, print_warn};
use crate::profiles::{ensure_dirs, get_active_profile, save_profile, Profile, ProfileMeta};

pub fn run(_verbose: bool) -> Result<()> {
    if get_active_profile()?.is_some() || crate::config::paths().profiles_dir.exists() {
        print_warn("ccswitch is already initialized.");
        let confirmed = confirm_prompt("Re-initialize?")?;
        if !confirmed {
            return Ok(());
        }
    }

    ensure_dirs()?;
    print_info("Created ~/.claude-switcher/");

    match read_keychain() {
        Ok(kc) => {
            print_info("Existing Claude Code credentials found in Keychain.");
            let mut name = String::new();
            print!("Save as profile name: ");
            use std::io::Write;
            std::io::stdout().flush()?;
            std::io::stdin().read_line(&mut name)?;
            let name = name.trim().to_string();

            if !name.is_empty() {
                let profile = Profile {
                    claude_ai_oauth: kc.claude_ai_oauth,
                    meta: ProfileMeta {
                        name: name.clone(),
                        last_synced: Utc::now(),
                    },
                };
                save_profile(&profile)?;
                crate::profiles::set_active_profile(&name)?;
                print_success(&format!("Profile \"{name}\" saved and set as active."));
            }
        }
        Err(_) => {
            print_info("No existing credentials found. Use `ccswitch add <name>` after logging in.");
        }
    }

    // Start daemon
    let binary = std::env::current_exe()?;
    match crate::daemon::launchd::install_plist(&binary) {
        Ok(()) => print_success("Daemon started."),
        Err(e) => print_warn(&format!("Could not start daemon: {e}")),
    }

    println!();
    println!("Next steps:");
    println!("  ccswitch add <name>   — save current credentials as a profile");
    println!("  ccswitch list         — list all profiles");
    println!("  ccswitch use <name>   — switch to a profile");

    Ok(())
}
