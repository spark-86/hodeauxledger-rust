use hodeauxledger_core::{Key, Rhex};

use crate::rhex::builder;

pub fn verifiy_failed(
    our_key: &Key,
    err: anyhow::Error,
    rhex: &Rhex,
) -> Result<Rhex, anyhow::Error> {
    let record_type = "error:verify_failed";
    let data = serde_json::json!({
        "failed_rhex": rhex,
        "error": err.to_string(),
    });
    let rhex = builder::build_rhex(&[0u8; 32], "", our_key, &[0u8; 32], record_type, data);
    Ok(rhex)
}
