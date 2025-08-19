use anyhow::{Context, anyhow};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

pub fn to_base64(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn from_base64(s: &str) -> Result<Vec<u8>, anyhow::Error> {
    URL_SAFE_NO_PAD
        .decode(s.trim())
        .with_context(|| "invalid base64 url (no padding)".to_string())
}

pub fn from_base64_to_32(s: &str) -> Result<[u8; 32], anyhow::Error> {
    let v = from_base64(s)?;
    <[u8; 32]>::try_from(v.as_slice()).map_err(|_| anyhow!("decoded value not 32 bytes"))
}

pub fn from_base64_to_64(s: &str) -> Result<[u8; 64], anyhow::Error> {
    let v = from_base64(s)?;
    <[u8; 64]>::try_from(v.as_slice()).map_err(|_| anyhow!("decoded value not 64 bytes"))
}
