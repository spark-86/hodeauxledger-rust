use anyhow::{Context, Result};
use hodeauxledger_core::rhex::intent::Intent;
use hodeauxledger_core::scope::authority::Authority;
use std::{fs, path::Path};

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

pub fn load_root_auth(path: &str) -> Result<Vec<Authority>> {
    if path.is_empty() {
        anyhow::bail!("empty path");
    }
    let p = Path::new(path);
    let data = fs::read(p)?;
    let root = serde_json::from_slice(&data)?;
    Ok(root)
}
