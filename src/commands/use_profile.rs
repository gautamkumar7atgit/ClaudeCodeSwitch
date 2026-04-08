use anyhow::Result;
use std::time::Instant;

use crate::keychain::write_keychain;
use crate::output::{print_success, print_verbose};
use crate::process::kill_claude_processes;
use crate::profiles::{load_profile, set_active_profile};

pub fn run(name: &str, verbose: bool) -> Result<()> {
    let start = Instant::now();

    let profile = load_profile(name)?;
    print_verbose(&format!("Loaded profile \"{name}\""), verbose);

    let killed = kill_claude_processes()?;
    if killed > 0 {
        print_verbose(&format!("Killed {killed} claude process(es)"), verbose);
    }

    write_keychain(&profile.claude_ai_oauth)?;
    print_verbose("Keychain updated", verbose);

    set_active_profile(name)?;

    let elapsed = start.elapsed().as_secs_f64();
    print_success(&format!("Switched to \"{name}\" in {elapsed:.1}s"));
    Ok(())
}
