use hodeauxledger_core::{Key, Rhex, policy::policy::Policy};

use crate::rhex::builder;

pub fn set(
    scope: &str,
    head: &[u8; 32],
    author_key: Key,
    usher_pk: &[u8; 32],
    policy: Policy,
) -> Result<Rhex, anyhow::Error> {
    let record_type = "policy:set";
    let data = policy.to_json();
    let rhex = builder::build_rhex(head, scope, &author_key, usher_pk, record_type, data);
    Ok(rhex)
}
