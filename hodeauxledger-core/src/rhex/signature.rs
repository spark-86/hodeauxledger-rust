use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Signature {
    /// 0=author, 1=usher, 2..=quorum
    pub sig_type: u8,
    #[serde_as(as = "Bytes")]
    pub public_key: [u8; 32],
    #[serde_as(as = "Bytes")]
    pub sig: [u8; 64],
}

impl Signature {
    pub fn new() -> Self {
        Self {
            sig_type: 0,
            public_key: [0u8; 32],
            sig: [0u8; 64],
        }
    }
}
