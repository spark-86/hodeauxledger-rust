use anyhow::{Context, Result};
use hodeauxledger_core::rhex::rhex::Rhex;
use std::{fs, path::PathBuf};

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

pub fn load_raw_rhex(path: &PathBuf) -> Result<Vec<u8>> {
    let data = fs::read(path)?;
    Ok(data)
}
