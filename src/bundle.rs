use anyhow::{bail, Context, Result};
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::profiles::Profile;

// Binary layout of the encrypted payload (before base64 encoding):
//   [0..4]   version  u32 little-endian  (= 1)
//   [4..36]  salt     32 bytes           (random, Argon2id KDF input)
//   [36..48] nonce    12 bytes           (random, AES-256-GCM nonce)
//   [48..]   ciphertext                  (AES-256-GCM of the JSON profile array)
const PAYLOAD_VERSION: u32 = 1;
const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const HEADER_LEN: usize = 4 + SALT_LEN + NONCE_LEN; // 48

/// Portable bundle written to a `.ccspack` file.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportBundle {
    /// Format version — always 1 for now.
    pub version: u32,
    /// When the bundle was created.
    pub created_at: DateTime<Utc>,
    /// Whether the profiles are encrypted.
    pub encrypted: bool,
    /// Base64-encoded encrypted payload (encrypted bundles only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    /// Plaintext profiles (unencrypted bundles only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<Vec<Profile>>,
}

/// Encrypt `profiles` with AES-256-GCM, key derived from `passphrase` via Argon2id.
/// Returns a base64 string suitable for storing in `ExportBundle::payload`.
pub fn encrypt_profiles(profiles: &[Profile], passphrase: &str) -> Result<String> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Key, Nonce,
    };
    use argon2::{Algorithm, Argon2, Params, Version};
    use rand::RngCore;
    use rand::rngs::OsRng;

    let plaintext = serde_json::to_vec(profiles).context("failed to serialize profiles")?;

    // Random salt and nonce
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);

    // Argon2id key derivation
    let params = Params::new(65536, 3, 1, None)
        .map_err(|e| anyhow::anyhow!("invalid Argon2 params: {e}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key_bytes = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key_bytes)
        .map_err(|e| anyhow::anyhow!("key derivation failed: {e}"))?;

    // AES-256-GCM encryption
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|_| anyhow::anyhow!("encryption failed"))?;

    // Pack into binary blob: version(4) || salt(32) || nonce(12) || ciphertext
    let mut packed = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    packed.extend_from_slice(&PAYLOAD_VERSION.to_le_bytes());
    packed.extend_from_slice(&salt);
    packed.extend_from_slice(&nonce_bytes);
    packed.extend_from_slice(&ciphertext);

    Ok(base64::engine::general_purpose::STANDARD.encode(&packed))
}

/// Decrypt a base64 payload produced by [`encrypt_profiles`] using the given passphrase.
pub fn decrypt_profiles(payload: &str, passphrase: &str) -> Result<Vec<Profile>> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Key, Nonce,
    };
    use argon2::{Algorithm, Argon2, Params, Version};

    let packed = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .context("invalid base64 in bundle payload")?;

    if packed.len() < HEADER_LEN {
        bail!("bundle payload is too short to be valid");
    }

    let _version = u32::from_le_bytes(packed[0..4].try_into().unwrap());
    let salt = &packed[4..4 + SALT_LEN];
    let nonce_bytes = &packed[4 + SALT_LEN..HEADER_LEN];
    let ciphertext = &packed[HEADER_LEN..];

    // Re-derive key
    let params = Params::new(65536, 3, 1, None)
        .map_err(|e| anyhow::anyhow!("invalid Argon2 params: {e}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key_bytes = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key_bytes)
        .map_err(|e| anyhow::anyhow!("key derivation failed: {e}"))?;

    // Decrypt
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("decryption failed — wrong passphrase?"))?;

    serde_json::from_slice(&plaintext).context("failed to parse profiles from decrypted bundle")
}

/// Write a bundle to disk atomically, chmod 600.
pub fn write_bundle(bundle: &ExportBundle, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(bundle).context("failed to serialize bundle")?;
    // Use a sibling .tmp file for the atomic write
    let tmp = path.with_extension("ccspack.tmp");
    fs::write(&tmp, json.as_bytes())
        .with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("failed to chmod {}", tmp.display()))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("failed to rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

/// Read and deserialize a bundle from disk.
pub fn read_bundle(path: &Path) -> Result<ExportBundle> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&data)
        .with_context(|| format!("failed to parse bundle: {}", path.display()))
}
