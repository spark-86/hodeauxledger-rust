use hodeauxledger_core::{GTClock, Key, Rhex};

use crate::rhex::builder;

pub fn ok(
    our_key: Key,
    senders_pk: &[u8; 32],
    data: serde_json::Value,
) -> Result<Rhex, anyhow::Error> {
    let record_type = "confirm:ok";
    let rhex = builder::build_rhex([0u8; 32], "", &our_key, *senders_pk, record_type, data);
    let clock = GTClock::new(0);
    let rhex = builder::usher_sign(&rhex, clock.now_micromarks_u64(), *senders_pk);
    Ok(rhex)
}
