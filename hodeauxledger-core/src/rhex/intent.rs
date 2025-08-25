use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Intent {
    pub previous_hash: [u8; 32],
    pub scope: String,
    pub nonce: String,
    pub author_public_key: [u8; 32],
    pub usher_public_key: [u8; 32],
    pub record_type: String,
    pub data: serde_json::Value,
}

impl Intent {
    /// Builder for a fresh intent (fills nonce and µmark time).
    pub fn new(
        previous_hash: [u8; 32],
        scope: &str,
        nonce: &str,
        author_pk: [u8; 32],
        usher_pk: [u8; 32],
        record_type: &str,
        data: serde_json::Value,
    ) -> Self {
        Self {
            previous_hash,
            scope: scope.to_string(),
            nonce: nonce.to_string(),
            author_public_key: author_pk,
            usher_public_key: usher_pk,
            record_type: record_type.to_string(),
            data,
        }
    }

    /// Canonical bytes used for signing/hashing (canonical CBOR).
    pub fn canonical_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(super::rhex::Rhex::to_stable_cbor(self)?)
    }
}
