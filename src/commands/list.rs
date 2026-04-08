use anyhow::Result;

use crate::output::{format_profile_row, print_info};
use crate::profiles::{get_active_profile, list_profiles};

pub fn run(verbose: bool) -> Result<()> {
    let mut profiles = list_profiles()?;
    if profiles.is_empty() {
        print_info("No profiles found.");
        return Ok(());
    }

    let active = get_active_profile()?;

    // Sort: active first, then alphabetical
    profiles.sort_by(|a, b| {
        let a_active = active.as_deref() == Some(a.meta.name.as_str());
        let b_active = active.as_deref() == Some(b.meta.name.as_str());
        b_active.cmp(&a_active).then(a.meta.name.cmp(&b.meta.name))
    });

    println!("Profiles:");
    for profile in &profiles {
        let is_active = active.as_deref() == Some(profile.meta.name.as_str());
        println!("{}", format_profile_row(profile, is_active, verbose));
    }

    Ok(())
}
