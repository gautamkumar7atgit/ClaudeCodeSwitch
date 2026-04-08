use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::config::{keychain_account, KEYCHAIN_SERVICE};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthCredentials {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Raw Keychain entry: may contain nested `claudeAiOauth` or be the creds object directly.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeychainCredentials {
    pub claude_ai_oauth: OAuthCredentials,
}

pub fn read_keychain() -> Result<KeychainCredentials> {
    let account = keychain_account();
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            &account,
            "-w",
        ])
        .output()
        .context("failed to run `security` CLI")?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        if code == 44 {
            bail!("keychain entry not found (exit 44)");
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("security CLI failed (exit {code}): {stderr}");
    }

    let raw = String::from_utf8(output.stdout).context("keychain output is not valid UTF-8")?;
    let raw = raw.trim();

    let creds: KeychainCredentials =
        serde_json::from_str(raw).context("failed to parse keychain JSON")?;
    Ok(creds)
}

pub fn write_keychain(creds: &OAuthCredentials) -> Result<()> {
    let account = keychain_account();
    let wrapped = serde_json::json!({ "claudeAiOauth": creds });
    let json = serde_json::to_string(&wrapped).context("failed to serialize credentials")?;

    let output = Command::new("security")
        .args([
            "add-generic-password",
            "-U", // update if exists
            "-s",
            KEYCHAIN_SERVICE,
            "-a",
            &account,
            "-w",
            &json,
        ])
        .output()
        .context("failed to run `security` CLI")?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("security CLI failed (exit {code}): {stderr}");
    }

    Ok(())
}

/// Returns the last 8 characters of the access token for display.
pub fn credential_fingerprint(creds: &OAuthCredentials) -> String {
    let token = &creds.access_token;
    if token.len() >= 8 {
        format!("...{}", &token[token.len() - 8..])
    } else {
        token.clone()
    }
}
