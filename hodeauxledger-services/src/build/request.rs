use crate::rhex::builder;
use hodeauxledger_core::{Key, Rhex};

pub fn head(scope: &str, author_sk: Key, usher_pk: &[u8; 32]) -> Result<Rhex, anyhow::Error> {
    let record_type = "request:head";
    let data = serde_json::json!({});
    let rhex = builder::build_rhex([0u8; 32], scope, &author_sk, *usher_pk, record_type, data);
    Ok(rhex)
}

pub fn rhex(
    scope: &str,
    author_sk: Key,
    usher_pk: &[u8; 32],
    data: serde_json::Value,
) -> Result<Rhex, anyhow::Error> {
    let record_type = "request:rhex";
    let rhex = builder::build_rhex([0u8; 32], scope, &author_sk, *usher_pk, record_type, data);
    Ok(rhex)
}
