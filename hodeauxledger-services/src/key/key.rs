use hodeauxledger_core::Key;

pub fn validate_key_grant(data: &serde_json::Value) -> Result<Key, anyhow::Error> {
    Ok(Key::new())
}
