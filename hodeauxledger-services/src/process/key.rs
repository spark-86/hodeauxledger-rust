use anyhow::{Result, anyhow};
use ed25519_dalek::VerifyingKey;
use hodeauxledger_core::{Key, Rhex, from_base64};
use hodeauxledger_io::cache::cache::Cache;
use std::convert::TryInto;

pub fn process_key_records(rhex: &Rhex, first_time: bool) -> Result<(), anyhow::Error> {
    match rhex.intent.record_type.as_str() {
        "🔑:🟢" | "key:grant" => grant(rhex, first_time),
        "🔑:🔴" | "key:revoke" => revoke(rhex, first_time),
        _ => {
            anyhow::bail!("invalid record type: {}", rhex.intent.record_type.as_str());
        }
    }
}

pub fn grant(rhex: &Rhex, _first_time: bool) -> Result<()> {
    let cache = Cache::connect("")?;
    let scope = &rhex.intent.scope;

    // roles: Option<Vec<String>>
    let roles: Option<Vec<String>> = rhex
        .intent
        .data
        .get("roles")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|val| val.as_str().map(|s| s.to_string()))
                .collect::<Vec<String>>()
        });

    // effective_micromark: Option<u64>
    let effective_micromark: Option<u64> = rhex
        .intent
        .data
        .get("effective_micromark")
        .and_then(|v| v.as_u64());

    // expires_micromark: Option<u64>
    let expires_micromark: Option<u64> = rhex
        .intent
        .data
        .get("expires_micromark")
        .and_then(|v| v.as_u64());

    // public_key (base64/bytes) → VerifyingKey
    let pk_b64 = rhex
        .intent
        .data
        .get("public_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("missing public_key"))?;

    let pk_bytes = from_base64(pk_b64)?; // Vec<u8>

    // must be 32 bytes
    let pk_arr: [u8; 32] = pk_bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("public_key must be 32 bytes"))?;

    let verifying_key =
        VerifyingKey::from_bytes(&pk_arr).map_err(|e| anyhow!("invalid public_key: {e}"))?;

    // Build the Key
    let key = Key {
        roles,
        sk: None,
        pk: Some(verifying_key),
        effective_micromark,
        expires_micromark,
        // add other fields if your Key struct has them (e.g., scope)
    };

    // Persist
    cache.cache_key(&key, scope)?;

    Ok(())
}
pub fn revoke(rhex: &Rhex, first_time: bool) -> Result<(), anyhow::Error> {
    let cache = Cache::connect("")?;
    let _ = rhex;
    let _ = first_time;
    let _ = cache;
    Ok(())
}
