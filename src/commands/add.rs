use anyhow::Result;
use chrono::Utc;

use crate::keychain::read_keychain;
use crate::output::{confirm_prompt, print_success};
use crate::profiles::{load_profile, save_profile, Profile, ProfileMeta};

pub fn run(name: &str, overwrite: bool, _verbose: bool) -> Result<()> {
    // Check for existing profile
    if load_profile(name).is_ok() && !overwrite {
        let confirmed =
            confirm_prompt(&format!("Profile \"{name}\" already exists. Overwrite?"))?;
        if !confirmed {
            return Ok(());
        }
    }

    let kc = read_keychain()?;

    let profile = Profile {
        claude_ai_oauth: kc.claude_ai_oauth,
        meta: ProfileMeta {
            name: name.to_string(),
            last_synced: Utc::now(),
        },
    };

    save_profile(&profile)?;
    print_success(&format!("Profile \"{name}\" saved."));
    Ok(())
}
