// --- signature.rs ---
use serde::{Deserialize, Serialize};
use serde_with::{Bytes, serde_as};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Signature {
    /// 0=author, 1=usher, 2=quorum (others reserved)
    #[serde(rename = "🤘", alias = "sig_type")]
    pub sig_type: u8,
    #[serde_as(as = "Bytes")]
    #[serde(rename = "🔓", alias = "public_key")]
    pub public_key: [u8; 32],
    #[serde_as(as = "Bytes")]
    #[serde(rename = "🖊️", alias = "sig")]
    pub sig: [u8; 64],
}

impl Signature {
    pub fn new() -> Self {
        Self {
            sig_type: 0,
            public_key: [0; 32],
            sig: [0; 64],
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SigType {
    Author = 0,
    Usher = 1,
    Quorum = 2,
}

impl TryFrom<u8> for SigType {
    type Error = anyhow::Error;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(SigType::Author),
            1 => Ok(SigType::Usher),
            2 => Ok(SigType::Quorum),
            _ => Err(anyhow::anyhow!("invalid signature type: {v}")),
        }
    }
}

impl From<SigType> for u8 {
    fn from(s: SigType) -> u8 {
        s as u8
    }
}
