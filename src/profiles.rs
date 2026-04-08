use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;

use crate::config::paths;
use crate::keychain::OAuthCredentials;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileMeta {
    pub name: String,
    pub last_synced: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub claude_ai_oauth: OAuthCredentials,
    #[serde(rename = "_meta")]
    pub meta: ProfileMeta,
}

pub fn ensure_dirs() -> Result<()> {
    let p = paths();
    fs::create_dir_all(&p.profiles_dir)
        .with_context(|| format!("failed to create profiles dir: {}", p.profiles_dir.display()))?;
    // chmod 700 the base dir
    let base = p.profiles_dir.parent().unwrap();
    fs::set_permissions(base, fs::Permissions::from_mode(0o700))
        .with_context(|| format!("failed to chmod {}", base.display()))?;
    Ok(())
}

pub fn list_profiles() -> Result<Vec<Profile>> {
    let p = paths();
    if !p.profiles_dir.exists() {
        return Ok(vec![]);
    }
    let mut profiles = Vec::new();
    for entry in fs::read_dir(&p.profiles_dir)
        .with_context(|| format!("failed to read profiles dir: {}", p.profiles_dir.display()))?
    {
        let entry = entry.context("failed to read directory entry")?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let data = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let profile: Profile = serde_json::from_str(&data)
                .with_context(|| format!("failed to parse {}", path.display()))?;
            profiles.push(profile);
        }
    }
    Ok(profiles)
}

pub fn load_profile(name: &str) -> Result<Profile> {
    let p = paths();
    let path = p.profiles_dir.join(format!("{name}.json"));
    if !path.exists() {
        bail!("profile not found: {name}");
    }
    let data = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let profile: Profile = serde_json::from_str(&data)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(profile)
}

pub fn save_profile(profile: &Profile) -> Result<()> {
    let p = paths();
    ensure_dirs()?;
    let path = p.profiles_dir.join(format!("{}.json", profile.meta.name));
    let tmp = p.profiles_dir.join(format!("{}.json.tmp", profile.meta.name));

    let json = serde_json::to_string_pretty(profile).context("failed to serialize profile")?;
    fs::write(&tmp, &json)
        .with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("failed to chmod {}", tmp.display()))?;
    fs::rename(&tmp, &path)
        .with_context(|| format!("failed to rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

pub fn delete_profile(name: &str) -> Result<()> {
    let p = paths();
    let path = p.profiles_dir.join(format!("{name}.json"));
    if !path.exists() {
        bail!("profile not found: {name}");
    }
    fs::remove_file(&path)
        .with_context(|| format!("failed to delete {}", path.display()))?;
    Ok(())
}

pub fn get_active_profile() -> Result<Option<String>> {
    let p = paths();
    if !p.active_file.exists() {
        return Ok(None);
    }
    let name = fs::read_to_string(&p.active_file)
        .with_context(|| format!("failed to read {}", p.active_file.display()))?;
    let name = name.trim().to_string();
    if name.is_empty() {
        Ok(None)
    } else {
        Ok(Some(name))
    }
}

pub fn set_active_profile(name: &str) -> Result<()> {
    let p = paths();
    ensure_dirs()?;
    let tmp = p.active_file.with_extension("tmp");
    fs::write(&tmp, name)
        .with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, &p.active_file)
        .with_context(|| format!("failed to rename to {}", p.active_file.display()))?;
    Ok(())
}

pub fn clear_active_profile() -> Result<()> {
    let p = paths();
    if p.active_file.exists() {
        fs::remove_file(&p.active_file)
            .with_context(|| format!("failed to remove {}", p.active_file.display()))?;
    }
    Ok(())
}
