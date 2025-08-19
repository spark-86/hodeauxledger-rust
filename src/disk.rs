use crate::rhex::{Intent, Rhex};
use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
use anyhow::{Context, Result, anyhow, bail};
use argon2::{Algorithm, Argon2, Params, Version};
use ed25519_dalek::SigningKey;
use rand::RngCore;
use std::{
    fs,
    path::{Path, PathBuf},
};
use zeroize::Zeroize;

const MAGIC: &[u8; 6] = b"HKYV1\0"; // Hodeaux Key, Version 1
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

/// File layout:
/// [0..6)  = MAGIC "HKYV1\0"
/// [6..22) = SALT (16B)
/// [22..34)= NONCE (12B)
/// [34.. ) = CIPHERTEXT (AEAD: key 32B plaintext -> 32B + 16B tag)

/// Derive a 32B AES key from password+salt using Argon2id (strong, portable).
fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    // Reasonable defaults; tune as you like (time/memory parallelism)
    let params =
        Params::new(19 * 1024, 2, 1, Some(32)).map_err(|_| anyhow::anyhow!("invalid params"))?; // 19 MiB, 2 iters, 1 lane → 32 bytes
    let a2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    // Argon2 crate works with PHC strings; we can hash into an output buffer:
    let mut out = [0u8; 32];
    let _ = a2
        .hash_password_into(password.as_bytes(), salt, &mut out)
        .map_err(|_| anyhow::anyhow!("password hash failed"));
    Ok(out)
}

/// Save an Ed25519 SigningKey encrypted with a password.
pub fn save_key(path: &Path, password: &str, signing_key: &SigningKey) -> Result<()> {
    if password.is_empty() {
        anyhow::bail!("empty password");
    }
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce);

    let mut aes_key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&aes_key).context("bad AES key")?;

    let key_bytes = signing_key.to_bytes(); // [u8;32]
    let ciphertext = cipher
        .encrypt(&nonce.into(), key_bytes.as_ref())
        .map_err(|_| anyhow::anyhow!("encryption failed"))?;

    // Assemble file
    let mut out = Vec::with_capacity(MAGIC.len() + SALT_LEN + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);

    // Best-effort zero secrets from memory
    aes_key.zeroize();

    // Atomic-ish write: write to temp then rename
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &out).with_context(|| format!("write temp key file {:?}", tmp))?;
    fs::rename(&tmp, path).with_context(|| format!("rename {:?} -> {:?}", tmp, path))?;
    Ok(())
}

/// Load an Ed25519 SigningKey by decrypting with password.
pub fn load_key(path: &str, password: &str) -> Result<SigningKey> {
    let data = fs::read(path).with_context(|| format!("read key file {}", path))?;
    if data.len() < MAGIC.len() + SALT_LEN + NONCE_LEN {
        anyhow::bail!("key file too short");
    }
    let (magic, rest) = data.split_at(MAGIC.len());
    if magic != MAGIC {
        anyhow::bail!("bad key magic/version");
    }
    let (salt, rest) = rest.split_at(SALT_LEN);
    let (nonce, ciphertext) = rest.split_at(NONCE_LEN);
    if ciphertext.len() < 16 {
        // must at least contain GCM tag
        anyhow::bail!("ciphertext truncated");
    }

    let mut aes_key = derive_key(password, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&aes_key).context("bad AES key")?;

    let plaintext = cipher
        .decrypt(nonce.into(), ciphertext)
        .map_err(|_| anyhow::anyhow!("decryption failed"))?;
    aes_key.zeroize();

    if plaintext.len() != 32 {
        anyhow::bail!("decrypted key wrong length");
    }
    let mut sk_bytes = [0u8; 32];
    sk_bytes.copy_from_slice(&plaintext);
    Ok(SigningKey::from_bytes(&sk_bytes))
}

pub fn save_rhex(path: &PathBuf, rhex: &Rhex) -> Result<()> {
    let tmp = path.with_extension("tmp");

    let v = serde_cbor::value::to_value(rhex)?;

    let rhex_bytes =
        serde_cbor::to_vec(&v).with_context(|| format!("serialize Rhex to CBOR for {:?}", tmp))?;

    fs::write(&tmp, rhex_bytes).with_context(|| format!("write temp Rhex file {:?}", tmp))?;
    fs::rename(&tmp, path).with_context(|| format!("rename {:?} -> {:?}", tmp, path))?;
    Ok(())
}

pub fn load_rhex(path: &PathBuf) -> Result<Rhex> {
    let data = fs::read(path)?;
    let rhex: Rhex = serde_cbor::from_slice(&data)
        .with_context(|| format!("deserialize Rhex from CBOR for {:?}", path))?;
    Ok(rhex)
}

pub fn load_scope(dir: &str) -> Result<Vec<Rhex>> {
    let base = Path::new(dir);
    let mut out = Vec::new();

    // 1) Load scope:genesis first
    let genesis_path = base.join("genesis.rhex");
    if !genesis_path.exists() {
        bail!("Missing genesis file: {}", genesis_path.display());
    }

    let curr = load_rhex(&genesis_path)?;
    let mut working_hash = curr
        .current_hash
        .ok_or_else(|| anyhow!("genesis has no current_hash"))?;

    out.push(curr);

    // 2) Walk forward: find the one file whose previous_hash == working_hash
    loop {
        // Try to find a child file in this directory.
        let mut found_child: Option<(PathBuf, Rhex)> = None;

        for entry in fs::read_dir(base)? {
            let entry = entry?;
            let p = entry.path();

            // skip non-files and genesis
            if !p.is_file() {
                continue;
            }
            if p.file_name().and_then(|s| s.to_str()) == Some("genesis.rhex") {
                continue;
            }
            if p.extension().and_then(|s| s.to_str()) != Some("rhex") {
                continue;
            }

            // Load candidate and check previous_hash
            let candidate = match load_rhex(&p) {
                Ok(x) => x,
                Err(_) => continue, // skip unreadables
            };

            // Only advance if this file says its previous_hash == current working_hash
            if candidate.intent.previous_hash == working_hash {
                found_child = Some((p.clone(), candidate));
                break;
            }
        }

        match found_child {
            Some((_path, child)) => {
                working_hash = child
                    .current_hash
                    .ok_or_else(|| anyhow!("child record missing current_hash"))?;
                out.push(child);
            }
            None => {
                // No child found → we've hit the head for this scope.
                break;
            }
        }
    }

    Ok(out)
}

pub fn load_json_data(path: &str) -> Result<serde_json::Value> {
    let data = fs::read(path)?;
    Ok(serde_json::from_slice(&data)?)
}

pub fn save_intent(path: &str, intent: &Intent) -> Result<()> {
    if path.is_empty() {
        anyhow::bail!("empty path");
    }
    let p = Path::new(path);
    let tmp = p.with_extension("tmp");

    let v = serde_cbor::value::to_value(intent)?;

    let rhex_bytes =
        serde_cbor::to_vec(&v).with_context(|| format!("serialize Rhex to CBOR for {:?}", tmp))?;

    fs::write(&tmp, rhex_bytes).with_context(|| format!("write temp Rhex file {:?}", tmp))?;
    fs::rename(&tmp, p).with_context(|| format!("rename {:?} -> {:?}", tmp, p))?;
    Ok(())
}
