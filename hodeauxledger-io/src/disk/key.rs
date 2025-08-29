use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
use anyhow::{Context, Result, bail};
use argon2::{Algorithm, Argon2, Params, Version};
use ed25519_dalek::SigningKey;
use rand::RngCore;
use std::{fs, path::Path};
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
    rand::rng().fill_bytes(&mut salt);
    rand::rng().fill_bytes(&mut nonce);

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

/// Saves a key as just a raw [u8; 32], needed for the usher's hot
/// key and other on the fly signing.
pub fn save_key_hot(path: &Path, signing_key: &SigningKey) -> Result<()> {
    let tmp = path.with_extension("tmp");
    let sk_bytes = signing_key.to_bytes();
    fs::write(&tmp, sk_bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Load an Ed25519 SigningKey by decrypting with password.
pub fn load_key(path: &Path, password: &str) -> Result<SigningKey> {
    let data = fs::read(path).with_context(|| format!("read key file {:?}", path.to_str()))?;
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

/// Loads a hot key for things like usher signing.
/// Hot files are just [u8; 32] streams.
pub fn load_key_hot(path: &Path) -> Result<[u8; 32]> {
    let keydata = fs::read(path)?;
    if keydata.len() != 32 {
        bail!("invalid key length: expected 32, got {}", keydata.len());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&keydata);
    Ok(key)
}
