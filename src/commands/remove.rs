use anyhow::Result;

use crate::output::{confirm_prompt, print_success, print_warn};
use crate::profiles::{clear_active_profile, delete_profile, get_active_profile, load_profile};

pub fn run(name: &str, force: bool, _verbose: bool) -> Result<()> {
    // Confirm profile exists
    load_profile(name)?;

    let active = get_active_profile()?;
    let is_active = active.as_deref() == Some(name);

    if is_active && !force {
        print_warn(&format!("\"{name}\" is the active profile."));
        let confirmed = confirm_prompt("Remove anyway?")?;
        if !confirmed {
            return Ok(());
        }
    }

    delete_profile(name)?;

    if is_active {
        clear_active_profile()?;
    }

    print_success(&format!("Profile \"{name}\" removed."));
    Ok(())
}
