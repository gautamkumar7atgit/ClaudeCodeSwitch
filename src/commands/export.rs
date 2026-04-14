use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

use crate::bundle::{self, ExportBundle};
use crate::output;
use crate::profiles;

pub fn run(
    name: Option<&str>,
    all: bool,
    output_path: Option<&Path>,
    no_encrypt: bool,
    verbose: bool,
) -> Result<()> {
    // Validate: must specify a name OR --all, not both, not neither
    if name.is_none() && !all {
        bail!("specify a profile name or use --all to export every profile");
    }
    if name.is_some() && all {
        bail!("cannot use both a profile name and --all");
    }

    // Load target profiles
    let profile_list = if all {
        let list = profiles::list_profiles()?;
        if list.is_empty() {
            bail!("no profiles found to export");
        }
        list
    } else {
        vec![profiles::load_profile(name.unwrap())?]
    };

    let n = profile_list.len();
    let label = if n == 1 { "profile" } else { "profiles" };

    output::print_verbose(&format!("exporting {n} {label}"), verbose);

    // Determine output path
    let out: PathBuf = match output_path {
        Some(p) => p.to_path_buf(),
        None => {
            if all {
                PathBuf::from("ccswitch-export.ccspack")
            } else {
                PathBuf::from(format!("{}.ccspack", name.unwrap()))
            }
        }
    };

    // Build bundle
    let bundle = if no_encrypt {
        output::print_warn("Writing PLAINTEXT bundle — credentials are NOT encrypted.");
        output::print_warn("Only share this file over fully trusted, private channels.");
        ExportBundle {
            version: 1,
            created_at: chrono::Utc::now(),
            encrypted: false,
            payload: None,
            profiles: Some(profile_list),
        }
    } else {
        // Prompt for passphrase (twice to confirm)
        let pass1 = rpassword::prompt_password("Passphrase: ")
            .map_err(|e| anyhow::anyhow!("failed to read passphrase: {e}"))?;
        let pass2 = rpassword::prompt_password("Confirm passphrase: ")
            .map_err(|e| anyhow::anyhow!("failed to read passphrase: {e}"))?;
        if pass1 != pass2 {
            bail!("passphrases do not match");
        }
        if pass1.is_empty() {
            bail!("passphrase must not be empty");
        }

        output::print_info("Encrypting bundle...");
        let payload = bundle::encrypt_profiles(&profile_list, &pass1)?;

        ExportBundle {
            version: 1,
            created_at: chrono::Utc::now(),
            encrypted: true,
            payload: Some(payload),
            profiles: None,
        }
    };

    bundle::write_bundle(&bundle, &out)?;

    output::print_success(&format!("Exported {n} {label} to {}", out.display()));
    if bundle.encrypted {
        output::print_info(
            "Share the passphrase with recipients separately — never in the same message as the file.",
        );
    }
    output::print_info(&format!("Recipients run:  ccswitch import {}", out.display()));

    Ok(())
}
