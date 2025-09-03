use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Default {
    #[serde(rename = "🥐", alias = "roles")]
    pub roles: Vec<String>,
    #[serde(rename = "🤝☝️", alias = "quorum_k")]
    pub quorum_k: u8,
    #[serde(rename = "🤝🥐", alias = "quorum_roles")]
    pub quorum_roles: Vec<String>,
    #[serde(rename = "↔️", alias = "rate_per_mark")]
    pub rate_per_mark: u64,
}
