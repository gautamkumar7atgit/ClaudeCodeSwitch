use anyhow::{bail, Result};
use std::path::Path;

use crate::bundle;
use crate::output;
use crate::profiles;

pub fn run(
    file: &Path,
    rename_as: Option<&str>,
    overwrite: bool,
    verbose: bool,
) -> Result<()> {
    let b = bundle::read_bundle(file)?;

    // Decrypt or unwrap plaintext profiles
    let mut profile_list = if b.encrypted {
        let payload = b
            .payload
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("bundle is marked encrypted but has no payload"))?;
        let passphrase = rpassword::prompt_password("Passphrase: ")
            .map_err(|e| anyhow::anyhow!("failed to read passphrase: {e}"))?;
        bundle::decrypt_profiles(payload, &passphrase)?
    } else {
        output::print_warn("Bundle is not encrypted — credentials are in plaintext.");
        b.profiles
            .ok_or_else(|| anyhow::anyhow!("bundle is marked plaintext but has no profiles"))?
    };

    if profile_list.is_empty() {
        bail!("bundle contains no profiles");
    }

    // --as rename: only valid for single-profile bundles
    if let Some(new_name) = rename_as {
        if profile_list.len() != 1 {
            bail!(
                "--as can only be used with single-profile bundles ({} profiles in this bundle)",
                profile_list.len()
            );
        }
        output::print_verbose(
            &format!(
                "renaming '{}' -> '{new_name}'",
                profile_list[0].meta.name
            ),
            verbose,
        );
        profile_list[0].meta.name = new_name.to_string();
    }

    output::print_verbose(
        &format!("importing {} profile(s)", profile_list.len()),
        verbose,
    );

    let mut imported = 0usize;
    let mut skipped = 0usize;

    for profile in &profile_list {
        let name = &profile.meta.name;

        // Check for an existing profile with the same name
        if profiles::load_profile(name).is_ok() {
            if !overwrite {
                let confirmed = output::confirm_prompt(&format!(
                    "Profile '{name}' already exists — overwrite?"
                ))?;
                if !confirmed {
                    output::print_info(&format!("Skipped '{name}'"));
                    skipped += 1;
                    continue;
                }
            }
        }

        profiles::save_profile(profile)?;
        output::print_verbose(&format!("saved profile '{name}'"), verbose);
        imported += 1;
    }

    if imported == 0 {
        output::print_warn("No profiles were imported.");
    } else {
        let label = if imported == 1 { "profile" } else { "profiles" };
        output::print_success(&format!("Imported {imported} {label}"));
        if skipped > 0 {
            output::print_info(&format!("{skipped} skipped (already exist)"));
        }
        output::print_info("Run `ccswitch list` to see all profiles.");
    }

    Ok(())
}
